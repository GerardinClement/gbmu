#![allow(unused_variables)]
#![allow(dead_code)]

pub mod block0;
pub mod block1;
pub mod block2;
pub mod block3;
pub mod block_prefix;
pub mod conditions;
pub mod flags_registers;
pub mod registers;
pub mod utils;

use crate::cpu::registers::{R8, R16, Registers};
use crate::memory::MemoryBus;
use std::fmt;

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub bus: MemoryBus,
}

impl Cpu {
    pub fn new(rom: Vec<u8>) -> Self {
        Cpu {
            registers: Registers::default(),
            bus: MemoryBus::new(rom),
            pc: 0x0100,
        }
    }

    fn execute_instruction(&mut self, instruction: u8) {
        let block_mask = 0b11000000;
        let block = (instruction & block_mask) >> 6;
        match block {
            0b00 => block0::match_instruction_block0(self, instruction),
            0b01 => block1::match_instruction_block1(self, instruction),
            0b10 => block2::match_instruction_block2(self, instruction),
            0b11 => block3::match_instruction_block3(self, instruction),
            _ => {
                println!("Unknown instruction block: {}", block);
                self.pc = self.pc.wrapping_add(1);
            }
        }
    }

    pub fn step(&mut self) {
        let instruction_byte = self.bus.read_byte(self.pc);
        // println!("pc: 0x{:02X}", self.pc);
        // println!("opcode: 0x{:02X}", instruction_byte);
        self.execute_instruction(instruction_byte);
        println!("{}", self);
        // println!(
        //     "---------------------------------------------------------------------------------------------"
        // )
    }

    pub fn get_r8_value(&self, register: R8) -> u8 {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.read_byte(addr)
            }
            _ => self.registers.get_r8_value(register),
        }
    }

    pub fn set_r8_value(&mut self, register: R8, value: u8) {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.write_byte(addr, value);
            }
            _ => self.registers.set_r8_value(register, value),
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.get_r8_value(R8::A),
            self.registers.get_flags_u8(),
            self.registers.get_r8_value(R8::B),
            self.registers.get_r8_value(R8::C),
            self.registers.get_r8_value(R8::D),
            self.registers.get_r8_value(R8::E),
            self.registers.get_r8_value(R8::H),
            self.registers.get_r8_value(R8::L),
            self.registers.get_sp(),
            self.pc,
            self.bus.read_byte(self.pc),
            self.bus.read_byte(self.pc.wrapping_add(1)),
            self.bus.read_byte(self.pc.wrapping_add(2)),
            self.bus.read_byte(self.pc.wrapping_add(3)),
        )
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            registers: Registers::default(),
            bus: MemoryBus::default(),
            pc: 0x0100,
        }
    }
}
