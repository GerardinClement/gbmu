#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::flags_registers::FlagsRegister;
use crate::memory::MemoryBus;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum R8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HLIndirect = 6,
    A = 7,
}

impl From<u8> for R8 {
    fn from(value: u8) -> Self {
        match value {
            0 => R8::B,
            1 => R8::C,
            2 => R8::D,
            3 => R8::E,
            4 => R8::H,
            5 => R8::L,
            6 => R8::HLIndirect,
            7 => R8::A,
            _ => panic!("Invalid value for R8: {}", value),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum R16 {
    BC = 0,
    DE = 1,
    HL = 2,
}

impl From<u8> for R16 {
    fn from(value: u8) -> Self {
        match value {
            0 => R16::BC,
            1 => R16::DE,
            2 => R16::HL,
            _ => panic!("Invalid value for R16: {}", value),
        }
    }
}

pub struct Registers {
    r8: [u8; 8],
    f: FlagsRegister,
}

impl Default for Registers{
    fn default() -> Self {
        Registers { 
            r8: [0x01, 0xB0, 0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D],
            f: FlagsRegister::default(),
        }
    }
}

impl Registers {
    pub fn set_r8_value(&mut self, target: R8, value: u8) {
        self.r8[target as usize] = value;
    }

    pub fn get_r8_value(&self, target: R8) -> u8 {
        self.r8[target as usize]
    }

    pub fn get_r16_value(&self, target: R16) -> u16 {
        match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
        }
    }

    pub fn set_r16_value(&mut self, target: R16, value: u16) {
        match target {
            R16::BC => self.set_bc(value),
            R16::DE => self.set_de(value),
            R16::HL => self.set_hl(value),
        }
    }

    pub fn set_r16_mem_value(&mut self, memory: &mut MemoryBus, target: R16, value: u8) {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
        };
        memory.write_byte(addr, value);
    }

    pub fn get_r16_mem_value(&self, memory: &MemoryBus, target: R16) -> u8 {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
        };
        memory.read_byte(addr)
    }

    pub fn add_to_r16(&mut self, target: R16, value: u16) {
        let r16_value = self.get_r16_value(target);
        let (new_value, did_overflow) = r16_value.overflowing_add(value);

        self.set_r16_value(target, new_value);
        let zero = value == 0;
        let subtract = false;
        let carry = did_overflow;
        let half_carry = (r16_value & 0xFFF) + (value & 0xFFF) > 0xFFF;
        self.f.set_all(zero, subtract, half_carry, carry);
    }

    pub fn rotate_left(&mut self, target: R8, carry: bool) {
        let r8 = self.get_r8_value(target);
        let outgoing_bit = (r8 & 0b10000000) >> 7;

        let bit: u8 = if carry {
            (r8 & 0b10000000) >> 7
        } else if self.f.get_carry() {
            1
        } else {
            0
        };

        let result = r8.rotate_left(1) | bit;

        self.set_r8_value(target, result);
        self.set_carry_flag(outgoing_bit == 1);
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }

    pub fn rotate_right(&mut self, target: R8, carry: bool) {
        let r8 = self.get_r8_value(target);
        let outgoing_bit = r8 & 0b00000001;

        let bit: u8 = if carry {
            r8 & 0b00000001
        } else if self.f.get_carry() {
            0b10000000
        } else {
            0
        };

        let result = if carry {
            r8.rotate_right(1)
        } else {
            r8.rotate_right(1) | bit
        };

        self.set_r8_value(target, result);
        self.set_carry_flag(outgoing_bit == 1);
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }

    pub fn set_zero_flag(&mut self, value: bool) {
        self.f.set_zero(value);
    }

    pub fn set_subtract_flag(&mut self, value: bool) {
        self.f.set_subtract(value);
    }

    pub fn set_half_carry_flag(&mut self, value: bool) {
        self.f.set_half_carry(value);
    }

    pub fn set_carry_flag(&mut self, value: bool) {
        self.f.set_carry(value);
    }

    pub fn get_zero_flag(&mut self) -> bool {
        self.f.get_zero()
    }

    pub fn get_subtract_flag(&mut self) -> bool {
        self.f.get_subtract()
    }

    pub fn get_half_carry_flag(&mut self) -> bool {
        self.f.get_half_carry()
    }

    pub fn get_carry_flag(&mut self) -> bool {
        self.f.get_carry()
    }

    pub fn get_a(&self) -> u8 {
        self.r8[R8::A as usize]
    }

    pub fn get_b(&self) -> u8 {
        self.r8[R8::B as usize]
    }

    pub fn get_c(&self) -> u8 {
        self.r8[R8::C as usize]
    }

    pub fn get_d(&self) -> u8 {
        self.r8[R8::D as usize]
    }

    pub fn get_e(&self) -> u8 {
        self.r8[R8::E as usize]
    }

    pub fn get_h(&self) -> u8 {
        self.r8[R8::H as usize]
    }

    pub fn get_l(&self) -> u8 {
        self.r8[R8::L as usize]
    }

    pub fn get_af(&self) -> u16 {
        let byte: u8 = u8::from(self.f.clone());
        ((self.r8[R8::A as usize] as u16) << 8) | (byte as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.r8[R8::A as usize] = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xFF) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        ((self.r8[R8::B as usize] as u16) << 8) | (self.r8[R8::C as usize] as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.r8[R8::B as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::C as usize] = (value & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.r8[R8::D as usize] as u16) << 8) | (self.r8[R8::E as usize] as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.r8[R8::D as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::E as usize] = (value & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.r8[R8::H as usize] as u16) << 8) | (self.r8[R8::L as usize] as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.r8[R8::H as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::L as usize] = (value & 0xFF) as u8;
    }
}
