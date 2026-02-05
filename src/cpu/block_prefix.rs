#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::Cpu;
use crate::cpu::block_prefix;
use crate::cpu::registers::R8;
use crate::cpu::utils;

const R8_MASK: u8 = 0b00000111;
const B3_MASK: u8 = 0b00111000;
const MIDDLE_3_BITS_MASK: u8 = 0b00111000;
const FIRST_2_BITS_MASK: u8 = 0b11000000;

const RST_VEC: [u8; 8] = [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38];

const INSTRUCTIONS_BLOCK_PREFIX1: [u8; 8] = [
    0b00000000, //rlc r8
    0b00001000, //rrc r8
    0b00010000, //rl r8
    0b00011000, //rr r8
    0b00100000, //sla r8
    0b00101000, //sra r8
    0b00110000, //swap r8
    0b00111000, //srl r8
];

const INSTRUCTIONS_BLOCK_PREFIX2: [u8; 3] = [
    0b01000000, //bit b3, r8
    0b10000000, //res b3, r8
    0b11000000, //set b3, r8
];

fn get_instruction_block_prefix(instruction: u8) -> u8 {
    if (instruction & FIRST_2_BITS_MASK) != 0 {
        let match_opcode: Vec<u8> = INSTRUCTIONS_BLOCK_PREFIX2
            .iter()
            .cloned()
            .filter(|&opcode| (instruction & FIRST_2_BITS_MASK) == (opcode & FIRST_2_BITS_MASK))
            .collect();

        if match_opcode.len() == 1 {
            return match_opcode[0];
        }
    }

    let match_opcode: Vec<u8> = INSTRUCTIONS_BLOCK_PREFIX1
        .iter()
        .cloned()
        .filter(|&opcode| (instruction & MIDDLE_3_BITS_MASK) == (opcode & MIDDLE_3_BITS_MASK))
        .collect();

    if match_opcode.len() == 1 {
        return match_opcode[0];
    }

    panic!("No unique instruction found for opcode: {instruction:#04x}");
}

pub fn execute_instruction_block_prefix(cpu: &mut Cpu, instruction: u8) -> u8 {
    let opcode = get_instruction_block_prefix(instruction);

    match opcode {
        0b00000000 => block_prefix::rlc_r8(cpu, instruction),
        0b00001000 => block_prefix::rrc_r8(cpu, instruction),
        0b00010000 => block_prefix::rl(cpu, instruction),
        0b00011000 => block_prefix::rr(cpu, instruction),
        0b00100000 => block_prefix::sla_r8(cpu, instruction),
        0b00101000 => block_prefix::sr_r8(cpu, instruction, true),
        0b00110000 => block_prefix::swap_r8(cpu, instruction),
        0b00111000 => block_prefix::sr_r8(cpu, instruction, false),
        0b01000000 => block_prefix::bit_b3_r8(cpu, instruction),
        0b10000000 => block_prefix::res_b3_r8(cpu, instruction),
        0b11000000 => block_prefix::set_b3_r8(cpu, instruction),
        _ => cpu.pc = cpu.pc.wrapping_add(1),
    }

    if (0b110 & instruction) == 0b110 {
        if (0b01000000 & instruction) == 0b01000000 {
            12
        } else {
            16
        }
    } else {
        8
    }
}

pub fn rlc_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.rotate_left(r8, false, false);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn rrc_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.rotate_right(r8, true, false);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn rl(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.rotate_left(r8, true, false);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn rr(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.rotate_right(r8, false, false);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn sla_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.shift_left(r8);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn sr_r8(cpu: &mut Cpu, instruction: u8, arithmetic: bool) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.shift_right(r8, arithmetic);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn swap_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    cpu.registers.swap(r8);
    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn bit_b3_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let b3 = (instruction & B3_MASK) >> 3;
    let r8_value = cpu.get_r8_value(r8);

    cpu.registers.set_zero_flag((r8_value & (1 << b3)) == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(true);

    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn res_b3_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let b3 = (instruction & B3_MASK) >> 3;
    let r8_value = cpu.get_r8_value(r8);

    let new_value = r8_value & !(1 << b3);

    cpu.set_r8_value(r8, new_value);

    cpu.pc = cpu.pc.wrapping_add(2);
}

pub fn set_b3_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let b3 = (instruction & B3_MASK) >> 3;
    let mut r8_value = cpu.get_r8_value(r8);

    r8_value |= 1 << b3;
    cpu.set_r8_value(r8, r8_value);

    cpu.pc = cpu.pc.wrapping_add(2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    #[test]
    fn test_rlc_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::B, 0b1000_0001);
        execute_instruction_block_prefix(&mut cpu, 0x00); // RLC B

        assert_eq!(cpu.get_r8_value(R8::B), 0b0000_0011);
        assert!(cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_rrc_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::B, 0b00000001);
        execute_instruction_block_prefix(&mut cpu, 0x08); // RRC C

        assert_eq!(cpu.get_r8_value(R8::B), 0b10000000);
        assert!(cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_rl_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::D, 0b01010101);
        cpu.registers.set_carry_flag(true);
        execute_instruction_block_prefix(&mut cpu, 0x12); // RL D

        assert_eq!(cpu.get_r8_value(R8::D), 0b10101011);
        assert!(!cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_rr_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::E, 0b0000_0001);
        cpu.registers.set_carry_flag(true);
        execute_instruction_block_prefix(&mut cpu, 0x1B); // RR E

        assert_eq!(cpu.get_r8_value(R8::E), 0b1000_0000);
        assert!(cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_sla_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::H, 0b1000_0000);
        execute_instruction_block_prefix(&mut cpu, 0x24); // SLA H

        assert_eq!(cpu.get_r8_value(R8::H), 0b0000_0000);
        assert!(cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_sra_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::L, 0b1000_0001);
        execute_instruction_block_prefix(&mut cpu, 0x2D); // SRA L

        assert_eq!(cpu.get_r8_value(R8::L), 0b1100_0000);
        assert!(cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_swap_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0xF0);
        execute_instruction_block_prefix(&mut cpu, 0x37); // SWAP A

        assert_eq!(cpu.get_r8_value(R8::A), 0x0F);
    }

    #[test]
    fn test_srl_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::B, 0b0000_0010);
        execute_instruction_block_prefix(&mut cpu, 0x38); // SRL B

        assert_eq!(cpu.get_r8_value(R8::B), 0b0000_0001);
        assert!(!cpu.registers.get_carry_flag());
    }

    #[test]
    fn test_bit_b3_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::D, 0b0000_1000);
        execute_instruction_block_prefix(&mut cpu, 0x5A); // BIT 3, D

        assert!(!cpu.registers.get_zero_flag());
    }

    #[test]
    fn test_res_b3_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::E, 0b0000_1010); // Valeur initiale : bit 3 est à 1
        execute_instruction_block_prefix(&mut cpu, 0x9B); // RES 3, E

        assert_eq!(cpu.get_r8_value(R8::E), 0b0000_0010); // Bit 3 doit être réinitialisé à 0
    }

    #[test]
    fn test_res_b3_r8_6_c() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::C, 0b0100_0000); // Valeur initiale : bit 6 est à 1
        execute_instruction_block_prefix(&mut cpu, 0xB1); // RES 6, C

        assert_eq!(cpu.get_r8_value(R8::C), 0b0000_0000); // Bit 6 doit être réinitialisé à 0
    }

    #[test]
    fn test_set_b3_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::H, 0b0000_0000);
        execute_instruction_block_prefix(&mut cpu, 0xDC); // SET 3, H

        assert_eq!(cpu.get_r8_value(R8::H), 0b0000_1000);
    }

    #[test]
    fn test_set_b3_r8_7_d() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::H, 0b0000_0000);
        execute_instruction_block_prefix(&mut cpu, 0xDC); // SET 3, H

        assert_eq!(cpu.get_r8_value(R8::H), 0b0000_1000);
    }
}
