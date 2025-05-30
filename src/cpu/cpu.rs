use std::fmt;
use crate::cpu::registers::{Registers, R8};

use crate::instructions::execute_instruction;
use crate::memory::MemoryBus;

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
			bus: MemoryBus::new(),
    		pc: 0,

		}
	}
}
