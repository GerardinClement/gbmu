#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::Cpu;
use crate::cpu::registers::R8;
use crate::cpu::utils;

const R16_MASK: u8 = 0b00110000;
const R8_MASK: u8 = 0b00111000;
const COND_MASK: u8 = 0b00011000;

const INSTRUCTIONS_BLOCK1: [u8; 2] = [
    0b01000000, // LD r8, r8
    0b01110110, // halt
];

/// GET the instruction based on the opcode and returns the corresponding instruction.
fn get_instruction_block1(instruction: u8) -> u8 {
    if INSTRUCTIONS_BLOCK1.contains(&instruction) {
        instruction
    } else {
        INSTRUCTIONS_BLOCK1[0]
    }
}

pub fn execute_instruction_block1(cpu: &mut Cpu, instruction: u8) {
    let opcode: u8 = get_instruction_block1(instruction);

    match opcode {
        0b01000000 => load_r8_r8(cpu, instruction),
        // implement halt
        _ => cpu.pc = cpu.pc.wrapping_add(1),
    }
}

fn load_r8_r8(cpu: &mut Cpu, instruction: u8) {
    let source: R8 = utils::convert_source_index_to_r8(instruction);
    let dest: R8 = utils::convert_dest_index_to_r8(instruction);

    let value = cpu.get_r8_value(source);

    cpu.set_r8_value(dest, value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    #[test]
    fn test_load_r8_r8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::B, 0x42);
        execute_instruction_block1(&mut cpu, 0x40); // LD B, B

        assert_eq!(cpu.get_r8_value(R8::B), 0x42);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_halt() {
        let mut cpu = Cpu::default();
        execute_instruction_block1(&mut cpu, 0x76); // HALT

        // Assuming HALT sets a specific state or flag, you can check it here.
        // For now, we just check the PC increment.
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_load_r8_r8_different_registers() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::C, 0x55);
        execute_instruction_block1(&mut cpu, 0x41); // LD B, C

        assert_eq!(cpu.get_r8_value(R8::B), 0x55);
        assert_eq!(cpu.pc, 0x0100 + 1);
    }
}
