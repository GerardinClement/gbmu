use std::fmt;


use crate::Registers;

enum Instruction {
	ADD(ArithmeticTarget),
}

impl Instruction {
	fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
		if prefixed {
			Instruction::from_byte_prefixed(byte)
		} else {
			Instruction::from_byte_not_prefixed(byte)
		}
	}

	fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
			_ => panic!("Instruction {} not set", byte)
		}
	}

	fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
			_ => panic!("Instruction {} not set", byte)
		}
	}
}

enum ArithmeticTarget {
	A, B, C, D, E, H, L,
}

pub struct CPU {
	pub registers: Registers,
	pub pc: u16,
	pub bus: MemoryBus,
}

impl CPU {
	fn step(&mut self) {
		let mut instruction_byte = self.bus.read_byte(self.pc);
		let prefixed = instruction_byte == 0xCB;
		if prefixed {
			instruction_byte = self.bus.read_byte(self.pc + 1)
		}

		let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
			self.execute(instruction)
		} else {
			let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, instruction_byte);
			panic!("Unkown instruction found for: {}", description)
		};

		self.pc = next_pc;
	}

	fn execute(&mut self, instruction: Instruction) -> u16 {
		match instruction {
			Instruction::ADD(target) => {
				match target {
					ArithmeticTarget::C => {
						let value = self.registers.c;
						self.registers.a = self.add(value);
						return self.pc.wrapping_add(1);
					},
					_ => panic!("target not covered")
				}
			}
			_ => panic!("Unknown execute instruction")
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
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Par exemple, tu affiches les registres et le PC :
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

struct MemoryBus {
    memory: [u8; 0xFFFF]
}


impl MemoryBus {
	fn read_byte(&self, address: u16) -> u8 {
		return self.memory[address as usize]
 	}
}
