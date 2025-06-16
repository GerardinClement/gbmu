use crate::cpu::Cpu;
use crate::cpu::conditions::Cond;
use crate::cpu::registers::{R8, R16, R16Mem};

pub const R16_MASK: u8 = 0b00110000;
pub const DEST_R8_MASK: u8 = 0b00111000;
pub const SOURCE_R8_MASK: u8 = 0b00000111;
pub const COND_MASK: u8 = 0b00011000;

pub fn get_imm16(cpu: &mut Cpu) -> u16 {
    let lsb = cpu.bus.borrow().read_byte(cpu.pc + 1) as u16;
    let msb = cpu.bus.borrow().read_byte(cpu.pc + 2) as u16;
    let imm16 = (msb << 8) | lsb;
    imm16
}

pub fn convert_index_to_r16(instruction: u8) -> R16 {
    let r16_index = (instruction & R16_MASK) >> 4;
    R16::from(r16_index)
}

pub fn convert_index_to_r16_mem(instruction: u8) -> R16Mem {
    let r16_index = (instruction & R16_MASK) >> 4;
    R16Mem::from(r16_index)
}

pub fn convert_dest_index_to_r8(instruction: u8) -> R8 {
    let r8_index: u8 = (instruction & DEST_R8_MASK) >> 3;
    R8::from(r8_index)
}

pub fn convert_source_index_to_r8(instruction: u8) -> R8 {
    let r8_index: u8 = instruction & SOURCE_R8_MASK;
    R8::from(r8_index)
}

pub fn convert_index_to_cond(instruction: u8) -> Cond {
    let cond_index: u8 = (instruction & COND_MASK) >> 3;
    Cond::from(cond_index)
}

pub fn modify_hl(cpu: &mut Cpu, r16_mem: R16Mem) {
    let value = cpu.registers.get_r16_value(R16::HL);

    if r16_mem == R16Mem::HLincrement {
        cpu.registers.set_r16_value(R16::HL, value.wrapping_add(1));
    }
    if r16_mem == R16Mem::HLdecrement {
        cpu.registers.set_r16_value(R16::HL, value.wrapping_sub(1));
    }
}
