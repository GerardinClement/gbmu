#![allow(unused_variables)]
#![allow(dead_code)]

use crate::{cpu::registers::Registers, mmu::mbc::Mbc};

#[derive(Debug)]
pub enum Cond {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
    None = 4,
}

impl From<u8> for Cond {
    fn from(value: u8) -> Self {
        match value {
            0 => Cond::NZ,
            1 => Cond::Z,
            2 => Cond::NC,
            3 => Cond::C,
            _ => Cond::None,
        }
    }
}

impl Cond {
    pub fn test(&self, registers: &mut Registers) -> bool {
        match self {
            Cond::NZ => !registers.get_zero_flag(),
            Cond::Z => registers.get_zero_flag(),
            Cond::NC => !registers.get_carry_flag(),
            Cond::C => registers.get_carry_flag(),
            Cond::None => true,
        }
    }
}
