#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::Cpu;
use crate::cpu::registers::R8;
use crate::cpu::utils;

const R16_MASK: u8 = 0b00110000;
const R8_MASK: u8 = 0b00111000;
const COND_MASK: u8 = 0b00011000;

const INSTRUCTIONS_BLOCK2: [u8; 8] = [
    0b10000000, //add a, r8
    0b10001000, //adc a, r8
    0b10010000, //sub a, r8
    0b10011000, //sbc a, r8
    0b10100000, //and a, r8
    0b10101000, //xor a, r8
    0b10110000, //or a, r8
    0b10111000, //cp a, r8
];

/// GET the instruction based on the opcode and returns the corresponding instruction.
fn get_instruction_block2(instruction: u8) -> u8 {
    let block2_mask: u8 = 0b00111000;

    let match_opcode: Vec<u8> = INSTRUCTIONS_BLOCK2
        .iter()
        .cloned()
        .filter(|&opcode| (instruction & block2_mask) == (opcode & block2_mask))
        .collect();

    if match_opcode.len() == 1 {
        match_opcode[0]
    } else {
        panic!(
            "No unique instruction found for opcode: {:#04x}",
            instruction
        );
    }
}

pub fn match_instruction_block2(cpu: &mut Cpu, instruction: u8) {
    let opcode = get_instruction_block2(instruction);

    match opcode {
        0b10000000 => add_a_r8(cpu, instruction, false),
        0b10001000 => add_a_r8(cpu, instruction, true),
        0b10010000 => sub_a_r8(cpu, instruction, false),
        0b10011000 => sub_a_r8(cpu, instruction, true),
        0b10100000 => and_a_r8(cpu, instruction),
        0b10101000 => xor_a_r8(cpu, instruction),
        0b10110000 => or_a_r8(cpu, instruction),
        0b10111000 => cp_a_r8(cpu, instruction),
        // implement halt
        _ => cpu.pc = cpu.pc.wrapping_add(1),
    }
}

fn add_a_r8(cpu: &mut Cpu, instruction: u8, with_carry: bool) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    cpu.registers.add_to_r8(R8::A, r8_value, with_carry);
    cpu.pc = cpu.pc.wrapping_add(1)
}

fn sub_a_r8(cpu: &mut Cpu, instruction: u8, with_carry: bool) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    cpu.registers.sub_to_r8(R8::A, r8_value, with_carry);
    cpu.pc = cpu.pc.wrapping_add(1)
}

fn and_a_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value & r8_value;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(true);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(1)
}

fn xor_a_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value ^ r8_value;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(1)
}

fn or_a_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value | r8_value;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(1)
}

fn cp_a_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);

    let r8_value = cpu.get_r8_value(r8);
    let a_value = cpu.get_r8_value(R8::A);

    let value = a_value - r8_value;

    cpu.registers.set_zero_flag(value == 0);
    cpu.registers.set_subtract_flag(true);
    cpu.registers
        .set_half_carry_flag((r8_value & 0x0F) < (value & 0x0F));
    cpu.registers.set_carry_flag(r8_value > a_value);

    cpu.pc = cpu.pc.wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    #[test]
    fn test_add_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x10);
        cpu.set_r8_value(R8::B, 0x20);
        match_instruction_block2(&mut cpu, 0x80); // ADD A, B

        assert_eq!(cpu.get_r8_value(R8::A), 0x30);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_adc_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x10);
        cpu.set_r8_value(R8::C, 0x20);
        cpu.registers.set_carry_flag(true);
        match_instruction_block2(&mut cpu, 0x49); // ADC A, C

        assert_eq!(cpu.get_r8_value(R8::A), 0x31);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_sub_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x30);
        cpu.set_r8_value(R8::C, 0x10);
        match_instruction_block2(&mut cpu, 0x91); // SUB A, C

        assert_eq!(cpu.get_r8_value(R8::A), 0x20);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_sbc_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x30);
        cpu.set_r8_value(R8::E, 0x10);
        cpu.registers.set_carry_flag(true);
        match_instruction_block2(&mut cpu, 0x9B); // SBC A, E

        assert_eq!(cpu.get_r8_value(R8::A), 0x1F);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_and_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.set_r8_value(R8::D, 0b1010);
        match_instruction_block2(&mut cpu, 0xA2); // AND A, D

        assert_eq!(cpu.get_r8_value(R8::A), 0b1000);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_xor_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.set_r8_value(R8::E, 0b1010);
        match_instruction_block2(&mut cpu, 0xAB); // XOR A, E

        assert_eq!(cpu.get_r8_value(R8::A), 0b0110);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_or_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.set_r8_value(R8::H, 0b1010);
        match_instruction_block2(&mut cpu, 0xB4); // OR A, H

        assert_eq!(cpu.get_r8_value(R8::A), 0b1110);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_cp_a_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x20);
        cpu.set_r8_value(R8::L, 0x20);
        match_instruction_block2(&mut cpu, 0xBD); // CP A, L

        assert!(cpu.registers.get_zero_flag());
        assert_eq!(cpu.pc, 0x0100 + 1);
    }
}
