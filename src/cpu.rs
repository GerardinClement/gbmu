use std::fmt;
use crate::Registers;

use crate::instructions::execute_instruction;
use crate::registers::R8;


pub struct CPU {
	pub registers: Registers,
	pub pc: u16,
	pub bus: MemoryBus,
}

impl CPU {
	fn step(&mut self) {
		let instruction_byte = self.bus.read_byte(self.pc);
		execute_instruction(self, instruction_byte);

		self.pc = self.pc.wrapping_add(1); // Implement PC
	}

	// pub fn add(&mut self, value: u8) -> u8 {
	// 	let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
	// 	self.registers.f.zero = new_value == 0;
	// 	self.registers.f.subtract = false;
	// 	self.registers.f.carry = did_overflow;
	// 	self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
	// 	new_value
	// }

	// pub fn jump(&mut self, value: u16) {
	// 	self.pc = value;
	// }

}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Registers:\nA: {:02X}, B: {:02X}, C: {:02X}, D: {:02X}, E: {:02X}, H: {:02X}, L: {:02X}\nPC: {:04X}",
			self.registers.get_r8_value(R8::A),
			self.registers.get_r8_value(R8::B),
			self.registers.get_r8_value(R8::C),
			self.registers.get_r8_value(R8::D),
			self.registers.get_r8_value(R8::E),
			self.registers.get_r8_value(R8::H),
			self.registers.get_r8_value(R8::L),
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
	pub fn read_byte(&self, address: u16) -> u8 {
		return self.memory[address as usize]
 	}

	pub fn write_byte(&mut self, address: u16, value: u8) {
		self.memory[address as usize] = value;
	}
}
