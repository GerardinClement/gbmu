use std::fmt;


use crate::registers::Registers;
use crate::instruction::*;

pub struct CPU {
	pub registers: Registers,
	pub pc: u16,
	pub bus: MemoryBus,
}

impl CPU {
	fn check_test(&mut self, test: JumpTest) -> bool {
		match test {
			JumpTest::Carry => {
				println!("Test carry");
				true
			},
			JumpTest::NotCarry => {
				println!("Test not carry");
				true
			},
			JumpTest::Zero => {
				println!("Test zero");
				true
			},
			JumpTest::NotZero => {
				println!("Test not zero");
				true
			},
			JumpTest::None => {
				println!("Test none");
				true
			},
		}
	}

	fn step(&mut self) {
		let mut instruction_byte = self.bus.read_byte(self.pc);
		let prefixed = instruction_byte == 0xCB;
		if prefixed {
			instruction_byte = self.bus.read_byte(self.pc + 1)
		}

		let next_pc = if let Some(instruction) = Instructions::from_byte(instruction_byte, prefixed) {
			self.execute(instruction)
		} else {
			let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, instruction_byte);
			panic!("Unkown instruction found for: {}", description)
		};

		self.pc = next_pc;
	}

	fn execute(&mut self, instruction: Instructions) -> u16 {
		match instruction {
			Instructions::ADD(target) => {
				match target {
					ArithmeticTarget::C => {
						let value = self.registers.c;
						self.registers.a = self.add(value);
						self.pc.wrapping_add(1)
					},
					_ => panic!("target not covered")
				}
			},
			Instructions::JP(condition, target) => {
				if !self.check_test(condition) {
					self.pc.wrapping_add(3)
				} else {
					match target {
						JumpTarget::HL => {
							let value = self.registers.get_hl();
							self.jump(value);
							self.pc
						},
						JumpTarget::A16 => {
							let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
							let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
							let value = (most_significant_byte << 8) | least_significant_byte;
							self.jump(value);
							self.pc
						}
						_ => panic!("target not covered")
					}
				}
			},
			Instructions::LD(target, source) => {
				let is_16_bit = match source {
					LoadSource::D16 => true,
					_ => false,
				};

				self.load(&source, target, is_16_bit);

				if is_16_bit {
					self.pc.wrapping_add(3)
				} else {
					match source {
						LoadSource::D8 => self.pc.wrapping_add(2),
						_	=> self.pc.wrapping_add(1),
					}
				}
			}
			_ => panic!("Unknown execute instruction")
		}
	}

	pub fn load(&mut self, source: &LoadSource, target: LoadTarget, is_16_bit: bool) {
		if is_16_bit {
			let source_value = match source {
				LoadSource::D16 => {
					let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
					let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
					let value = (most_significant_byte << 8) | least_significant_byte;
					value
				},
				_ => { panic!("source doesn't exist") }
			};
			match target {
				LoadTarget::HL => self.registers.set_hl(source_value),
				LoadTarget::BC => self.registers.set_bc(source_value),
				LoadTarget::DE => self.registers.set_de(source_value),
				_ => { panic!("target doesn't exist") }
			};
		} else {
			let source_value = match source {
				LoadSource::A => self.registers.a,
				LoadSource::B => self.registers.b,
				LoadSource::C => self.registers.c,
				LoadSource::D => self.registers.d,
				LoadSource::E => self.registers.e,
				LoadSource::H => self.registers.h,
				LoadSource::L => self.registers.l,
				LoadSource::HL => self.bus.read_byte(self.registers.get_hl()),
				LoadSource::BC => self.bus.read_byte(self.registers.get_bc()),
				LoadSource::DE => self.bus.read_byte(self.registers.get_de()),
				LoadSource::HLI => {
					let value = self.registers.get_hl();
					let mut byte = self.bus.read_byte(value);
					byte = byte.wrapping_add(1);
					byte
				},
				LoadSource::HLD => {
					let value = self.registers.get_hl();
					let mut byte = self.bus.read_byte(value);
					byte = byte.wrapping_sub(1);
					byte
				},
				LoadSource::D8 => {
					let value = self.bus.read_byte(self.pc + 1);
					value
				},
				_ => { panic!("source doesn't exist") }
			};
			match target {
				LoadTarget::A => self.registers.a = source_value,
				LoadTarget::B => self.registers.b = source_value,
				LoadTarget::C => self.registers.c = source_value,
				LoadTarget::D => self.registers.d = source_value,
				LoadTarget::E => self.registers.e = source_value,
				LoadTarget::H => self.registers.h = source_value,
				LoadTarget::L => self.registers.l = source_value,
				LoadTarget::HL => {
					let value = self.registers.get_hl();
					self.bus.memory[value as usize] = source_value;
				},
				LoadTarget::BC => {
					let value = self.registers.get_bc();
					self.bus.memory[value as usize] = source_value;
				},
				LoadTarget::DE => {
					let value = self.registers.get_de();
					self.bus.memory[value as usize] = source_value;
				},
				LoadTarget::HLI => {
					let value = self.registers.get_hl();
					self.bus.memory[value as usize] = source_value;
					self.registers.set_hl(value.wrapping_add(1));
				},
				LoadTarget::HLD => {
					let value = self.registers.get_hl();
					self.bus.memory[value as usize] = source_value;
					self.registers.set_hl(value.wrapping_sub(1));
				},
				_ => { panic!("target doesn't exist") }
			};
		}
	}

	pub fn add(&mut self, value: u8) -> u8 {
		let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
		self.registers.f.zero = new_value == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = did_overflow;
		self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
		new_value
	}

	pub fn jump(&mut self, value: u16) {
		self.pc = value;
	}

}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Registers:\nA: {:02X}, B: {:02X}, C: {:02X}, D: {:02X}, E: {:02X}, H: {:02X}, L: {:02X}\nPC: {:04X}",
            self.registers.a,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.pc,
        )
    }
}


impl Default for CPU {
	fn default() -> Self {
		CPU {
			registers: Registers::default(),
			bus: MemoryBus { memory: [0; 0xFFFF]},
    		pc: 0,

		}
	}
}

pub struct MemoryBus {
    pub memory: [u8; 0xFFFF]
}


impl MemoryBus {
	fn read_byte(&self, address: u16) -> u8 {
		return self.memory[address as usize]
 	}
}
