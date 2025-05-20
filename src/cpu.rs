mod Registers;

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
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B))
		}
	}

	fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
		}
	}
}

enum ArithmeticTarget {
	A, B, C, D, E, H, L,
}

pub struct CPU {
	registers: Registers,
	pc: u16,
	bus: MemoryBus,
}

impl CPU {
	fn step(&mut self) {
		let mut instruction_byte = self.bus.read_byte(self.pc);
		if instruction_byte == 0xCB {
			instruction_byte = self.bus.read_byte(self.pc + 1)
		}

		let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
			return self.execute(instruction);
		} else {
			let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, instruction_byte);
			panic!("Unkown instruction found for: {}", description);
			return description;
		};

		self.pc = next_pc;
	}

	fn execute(&mut self, instruction: Instruction) {
		match instruction {
			Instruction::ADD(target) => {
				match target {
					ArithmeticTarget::C => {
						let value = self.registers.c;
						self.registers.a = self.add(value);
						return self.pc.wrapping_add(1);
					}
				}
			}
			_ => panic!("Unknown execute instruction")
		}
	}

	fn add(&mut self, value: u8) -> u8 {
		let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
		self.registers.f.zero = new_value == 0;
		self.registers.f.substract = false;
		self.registers.f.carry = did_overflow;
		self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
		return new_value;
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
