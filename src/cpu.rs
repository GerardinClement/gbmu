#![allow(unused_variables)]
#![allow(dead_code)]

pub mod block0;
pub mod conditions;
pub mod flags_registers;
pub mod registers;

use crate::cpu::registers::{R8, R16, Registers};
use crate::memory::MemoryBus;
use std::fmt;

pub struct Cpu {
    pub registers: Registers,
    pub sp: u16,
    pub pc: u16,
    pub bus: MemoryBus,
}

impl Cpu {
    fn execute_instruction(&mut self, instruction: u8) {
        let block_mask = 0b11000000;
        let block = (instruction & block_mask) >> 6;
        match block {
            0b00 => block0::match_instruction_block0(self, instruction),
            // TODO Add more blocks here as needed
            _ => panic!("Unknown instruction block: {}", block),
        }
    }

    fn step(&mut self) {
        let instruction_byte = self.bus.read_byte(self.pc);
        self.execute_instruction(instruction_byte);

        self.pc = self.pc.wrapping_add(1);
    }

    pub fn get_r8_value(&self, r: R8) -> u8 {
        match r {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.read_byte(addr)
            }
            _ => self.registers.get_r8_value(r),
        }
    }

    pub fn set_r8_value(&mut self, r: R8, value: u8) {
        match r {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.write_byte(addr, value);
            }
            _ => self.registers.set_r8_value(r, value),
        }
    }
}

impl fmt::Display for Cpu {
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

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            registers: Registers::default(),
            bus: MemoryBus::new(vec![0; 0x4000]),
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }
}
