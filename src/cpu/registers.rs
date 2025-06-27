#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::conditions::Cond;
use crate::cpu::flags_registers::FlagsRegister;
use crate::mmu::bus::MemoryBus;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum R16 {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3,
}

impl From<u8> for R16 {
    fn from(value: u8) -> Self {
        match value {
            0 => R16::BC,
            1 => R16::DE,
            2 => R16::HL,
            3 => R16::SP,
            _ => panic!("Invalid value for R16: {}", value),
        }
    }
}

impl From<R16Mem> for R16 {
    fn from(value: R16Mem) -> Self {
        match value {
            R16Mem::BC => R16::BC,
            R16Mem::DE => R16::DE,
            R16Mem::HLincrement | R16Mem::HLdecrement => R16::HL,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum R16Mem {
    BC = 0,
    DE = 1,
    HLincrement = 2,
    HLdecrement = 3,
}

impl From<u8> for R16Mem {
    fn from(value: u8) -> Self {
        match value {
            0 => R16Mem::BC,
            1 => R16Mem::DE,
            2 => R16Mem::HLincrement,
            3 => R16Mem::HLdecrement,
            _ => panic!("Invalid value for R16Mem: {}", value),
        }
    }
}

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
pub struct Registers {
    r8: [u8; 8],
    sp: u16,
    f: FlagsRegister,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            r8: [0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D, 0x01, 0x01],
            sp: 0xFFFE,
            f: FlagsRegister::default(),
        }
    }
}

impl Registers {
    pub fn get_flags(&self) -> FlagsRegister {
        self.f.clone()
    }

    pub fn get_flags_u8(&self) -> u8 {
        u8::from(self.f.clone())
    }

    pub fn set_r8_value(&mut self, target: R8, value: u8) {
        self.r8[target as usize] = value;
    }

    pub fn get_r8_value(&self, target: R8) -> u8 {
        self.r8[target as usize]
    }

    pub fn add_to_r8(&mut self, target: R8, value: u8, with_carry: bool) {
        let target_value = self.r8[target as usize];
        let carry_in = if with_carry && self.get_carry_flag() {
            1
        } else {
            0
        };

        let (intermediate, carry1) = target_value.overflowing_add(value);
        let (result, carry2) = intermediate.overflowing_add(carry_in);

        self.r8[target as usize] = result;

        let half_carry = ((target_value & 0x0F) + (value & 0x0F) + carry_in) > 0x0F;
        let carry = carry1 || carry2;

        self.f.set_all(result == 0, false, half_carry, carry);
    }

    pub fn sub_to_r8(&mut self, target: R8, value: u8, with_carry: bool) {
        let original = self.r8[target as usize];
        let carry = if with_carry && self.get_carry_flag() {
            1
        } else {
            0
        };

        let (intermediate, carry1) = original.overflowing_sub(value);
        let (result, carry2) = intermediate.overflowing_sub(carry);

        self.r8[target as usize] = result;

        let zero = result == 0;
        let subtract = true;
        let half_carry = ((original & 0x0F)
            .wrapping_sub(value & 0x0F)
            .wrapping_sub(carry))
            & 0x10
            != 0;
        let carry = carry1 || carry2;

        self.f.set_all(zero, subtract, half_carry, carry);
    }
    
    pub fn get_r16_value(&self, target: R16) -> u16 {
        match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            R16::SP => self.sp,
        }
    }

    pub fn set_r16_value(&mut self, target: R16, value: u16) {
        match target {
            R16::BC => self.set_bc(value),
            R16::DE => self.set_de(value),
            R16::HL => self.set_hl(value),
            R16::SP => self.sp = value,
        }
    }

    pub fn set_r16_mem_value(&mut self, memory: &mut MemoryBus, target: R16, value: u8) {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            _ => panic!("Cannot set memory value for SP directly"),
        };
        memory.write_byte(addr, value);
    }

    pub fn get_r16_mem_value(&self, memory: &MemoryBus, target: R16) -> u8 {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            _ => panic!("Cannot get memory value for SP directly"),
        };
        memory.read_byte(addr)
    }

    pub fn add_to_r16(&mut self, target: R16, value: u16) {
        let r16_value = self.get_r16_value(target);
        let (new_value, did_overflow) = r16_value.overflowing_add(value);

        self.set_r16_value(target, new_value);
        let zero = self.get_zero_flag();
        let subtract = false;
        let half_carry = (r16_value & 0xFFF) + (value & 0xFFF) > 0xFFF;
        let carry = did_overflow;
        self.f.set_all(zero, subtract, half_carry, carry);
    }

    pub fn add_sp_i8(&mut self, offset: i8) {
        let sp = self.sp;
        let offset_u16 = offset as i16 as u16; // pour faire l'addition signée
        let result = sp.wrapping_add(offset_u16);

        // Flags
        self.set_zero_flag(false);
        self.set_subtract_flag(false);

        self.set_half_carry_flag(((sp & 0xF) + (offset_u16 & 0xF)) > 0xF);
        self.set_carry_flag(((sp & 0xFF) + (offset_u16 & 0xFF)) > 0xFF);

        self.sp = result;
    }

    pub fn rotate_left(&mut self, target: R8, through_carry: bool) {
        let value = self.get_r8_value(target);
        let old_carry = if self.f.get_carry() { 1 } else { 0 };
        let bit7 = (value & 0x80) >> 7;

        let result = if through_carry {
            let rotated = value << 1;
            (rotated & 0b1111_1110) | old_carry
        } else {
            value.rotate_left(1)
        };

        self.set_r8_value(target, result);
        self.set_carry_flag(bit7 == 1);

        // Z flag est forcé à 0 pour A dans RLA/RLCA
        if target == R8::A {
            self.set_zero_flag(false);
        } else {
            self.set_zero_flag(result == 0);
        }

        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }
    
    

    pub fn rotate_right(&mut self, target: R8, circular: bool) {
        let r8 = self.get_r8_value(target);
        let old_carry = self.f.get_carry();
        let outgoing_bit = r8 & 0b00000001 != 0;

        let result = if circular {
            r8.rotate_right(1) // RRC
        } else {
            (r8 >> 1) | if old_carry { 0x80 } else { 0x00 } // RR
        };

        let zero = result == 0 && target != R8::A;
        self.set_r8_value(target, result);
        self.set_carry_flag(outgoing_bit);
        self.set_zero_flag(zero);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }

    pub fn shift_left(&mut self, target: R8) {
        let r8 = self.get_r8_value(target);
        let outgoing_bit = (r8 & 0b10000000) >> 7;

        let result = r8 << 1;
        self.set_r8_value(target, result);
        self.set_carry_flag(outgoing_bit == 1);
        self.set_zero_flag(result == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }

    pub fn shift_right(&mut self, target: R8, arithmetic: bool) {
        let r8 = self.get_r8_value(target);
        let outgoing_bit = r8 & 0b00000001;

        let result = if arithmetic {
            (r8 as i8 >> 1) as u8
        } else {
            r8 >> 1
        };
        self.set_r8_value(target, result);
        self.set_carry_flag(outgoing_bit == 1);
        self.set_zero_flag(result == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
    }

    pub fn swap(&mut self, target: R8) {
        let r8 = self.get_r8_value(target);
        let high_nibble = (r8 & 0xF0) >> 4;
        let low_nibble = (r8 & 0x0F) << 4;

        let result = low_nibble | high_nibble;
        self.set_r8_value(target, result);
        self.set_zero_flag(result == 0);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        self.set_carry_flag(false);
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

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    pub fn push_sp(&mut self, bus: &mut MemoryBus, value: u16) {
        let low = (value & 0x00FF) as u8;
        let high = (value >> 8) as u8;
        self.sp = self.sp.wrapping_sub(1);
        bus.write_byte(self.sp, high);
        self.sp = self.sp.wrapping_sub(1);
        bus.write_byte(self.sp, low);
    }

    pub fn pop_sp(&mut self, bus: &MemoryBus) -> u16 {
        let low = bus.read_byte(self.sp) as u16;
        let high = bus.read_byte(self.sp.wrapping_add(1)) as u16;
        self.sp = self.sp.wrapping_add(2);
        (high << 8) | low
    }

    pub fn check_condition(&mut self, cond: Cond) -> bool {
        match cond {
            Cond::NZ => !self.get_zero_flag(),  // NZ
            Cond::Z => self.get_zero_flag(),    // Z
            Cond::NC => !self.get_carry_flag(), // NC
            Cond::C => self.get_carry_flag(),   // C
            Cond::None => true,                 // None
        }
    }
}
