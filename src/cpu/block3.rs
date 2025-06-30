#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::Cpu;
use crate::cpu::block_prefix;
use crate::cpu::conditions::Cond;
use crate::cpu::registers::{R8, R16};
use crate::cpu::utils;

const R16STK_MASK: u8 = 0b00110000;
const TGT3_MASK: u8 = 0b00111000;
const COND_MASK: u8 = 0b00011000;

const RST_VEC: [u8; 8] = [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38];

const INSTRUCTIONS_BLOCK3: [u8; 29] = [
    0b11000110, //add a, imm8
    0b11001110, //adc a, imm8
    0b11010110, //sub a, imm8
    0b11011110, //sbc a, imm8
    0b11100110, //and a, imm8
    0b11101110, //xor a, imm8
    0b11110110, //or a, imm8
    0b11111110, //cp a, imm8
    0b11000000, //ret cond
    0b11001001, //ret
    0b11011001, //reti
    0b11000010, //jp cond, imm16
    0b11000011, //jp imm16
    0b11101001, //jp hl
    0b11000100, //call cond, imm16
    0b11001101, //call imm16
    0b11000111, //rst tgt3
    0b11001011, //prefix
    0b11100010, //ldh [c], a
    0b11100000, //ldh [imm8], a
    0b11101010, //ld [imm16], a
    0b11110010, //ldh a, [c]
    0b11110000, //ldh a, [imm8]
    0b11111010, //ld a, [imm16]
    0b11101000, //add sp, imm8
    0b11111000, //ld hl, sp + imm8
    0b11110001, //pop af
    0b11110101, //push af
    0b11111001, //ld sp, hl
];

const INSTRUCTION_STACK_BLOCK3: [u8; 2] = [
    0b11000001, //pop r16stk
    0b11000101, //push r16stk
];

const INSTRUCTION_INTERRUPT: [u8; 2] = [
    0b11110011, //di
    0b11111011, //ei
];

fn check_stack_stk16_instruction(instruction: u8) -> u8 {
    let stack_mask = 0b00001111;

    let match_stack_opcode: Vec<u8> = INSTRUCTION_STACK_BLOCK3
        .iter()
        .cloned()
        .filter(|&opcode| (instruction & stack_mask) == (opcode & stack_mask))
        .collect();

    if match_stack_opcode.len() == 1 {
        match_stack_opcode[0]
    } else {
        0
    }
}

fn get_instruction_block3(instruction: u8) -> u8 {
    let block3_mask = 0b00000111;
    let cond_mask = 0b11100000;

    if INSTRUCTIONS_BLOCK3.contains(&instruction) || INSTRUCTION_INTERRUPT.contains(&instruction) {
        return instruction;
    }

    let stack_instruction = check_stack_stk16_instruction(instruction);
    if stack_instruction != 0 {
        return stack_instruction;
    }

    let match_opcode: Vec<u8> = INSTRUCTIONS_BLOCK3
        .iter()
        .cloned()
        .filter(|&opcode| (instruction & block3_mask) == (opcode & block3_mask))
        .collect();

    if match_opcode.len() == 1 {
        return match_opcode[0];
    }

    let match_cond_opcode: Vec<u8> = match_opcode
        .into_iter()
        .filter(|&opcode| (instruction & cond_mask) == (opcode & cond_mask))
        .collect();

    if match_cond_opcode.len() == 1 {
        match_cond_opcode[0]
    } else {
        panic!(
            "No unique instruction found for opcode: {:#04x}",
            instruction
        );
    }
}

pub fn execute_instruction_block3(cpu: &mut Cpu, instruction: u8) {
    let opcode = get_instruction_block3(instruction);

    match opcode {
        0b11000110 => add_a_imm8(cpu, false),       // add a, imm8
        0b11001110 => add_a_imm8(cpu, true),        // adc a, imm8
        0b11010110 => sub_a_imm8(cpu, false),       // sub a, imm8
        0b11011110 => sub_a_imm8(cpu, true),        // sbc a, imm8
        0b11100110 => and_a_imm8(cpu),              // and a, imm8
        0b11101110 => xor_a_imm8(cpu),              // xor a, imm8
        0b11110110 => or_a_imm8(cpu),               // or a, imm8
        0b11111110 => cp_a_imm8(cpu),               // cp a, imm8
        0b11000000 => ret(cpu, instruction, true),  // ret cond
        0b11001001 => ret(cpu, instruction, false), // ret
        // TODO: implement RETI
        0b11000010 => jp_imm16(cpu, instruction, true), // jp cond, imm16
        0b11000011 => jp_imm16(cpu, instruction, false), // jp imm16
        0b11101001 => jp_hl(cpu),                       // jp hl
        0b11000100 => call_imm16(cpu, instruction, true), // call cond, imm16
        0b11001101 => call_imm16(cpu, instruction, false), // call imm16
        0b11000111 => rst_tgt3(cpu, instruction),       // rst tgt3
        0b11000001 => pop_r16(cpu, instruction),        // pop r16stk
        0b11110001 => pop_af(cpu),                      // pop af
        0b11000101 => push_r16(cpu, instruction),       // push r16stk
        0b11110101 => push_af(cpu),                     // push af
        0b11001011 => prefix(cpu),                      // prefix
        0b11100010 => ldh_c_a(cpu),                     // ldh [c], a
        0b11100000 => ldh_imm8_a(cpu),                  // ldh [imm8], a
        0b11101010 => ld_imm16_a(cpu),                  // ld [imm16], a
        0b11110010 => ldh_a_c(cpu),                     // ldh a, [c]
        0b11110000 => ldh_a_imm8(cpu),                  // ldh a, [imm8]
        0b11111010 => ld_a_imm16(cpu),                  // ld a, [imm16]
        0b11101000 => add_sp_imm8(cpu),                 // add sp, imm8
        0b11111000 => ld_hl_sp_add_imm8(cpu),           // ld hl, sp + imm8
        0b11111001 => ld_sp_hl(cpu),                    // ld sp, hl
        // TODO: implement DI
        // TODO: implement EI
        _ => cpu.pc = cpu.pc.wrapping_add(1),
    }
}

fn add_a_imm8(cpu: &mut Cpu, with_carry: bool) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);

    cpu.registers.add_to_r8(R8::A, imm8, with_carry);
    cpu.pc = cpu.pc.wrapping_add(2);
}

fn sub_a_imm8(cpu: &mut Cpu, with_carry: bool) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);

    cpu.registers.sub_to_r8(R8::A, imm8, with_carry);
    cpu.pc = cpu.pc.wrapping_add(2)
}

fn and_a_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value & imm8;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(true);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(2)
}

fn xor_a_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value ^ imm8;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(2)
}

fn or_a_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);
    let a_value = cpu.get_r8_value(R8::A);

    let new_value = a_value | imm8;
    cpu.set_r8_value(R8::A, new_value);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(false);

    cpu.pc = cpu.pc.wrapping_add(2);
}

fn cp_a_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);
    let a_value = cpu.get_r8_value(R8::A);

    let value = a_value.wrapping_sub(imm8);

    cpu.registers.set_zero_flag(value == 0);
    cpu.registers.set_subtract_flag(true);
    cpu.registers
        .set_half_carry_flag((a_value & 0x0F) < (value & 0x0F));
    cpu.registers.set_carry_flag(a_value < imm8);

    cpu.pc = cpu.pc.wrapping_add(2);
}

fn ret(cpu: &mut Cpu, instruction: u8, with_cond: bool) {
    let cond = if with_cond {
        utils::convert_index_to_cond(instruction)
    } else {
        Cond::None
    };

    if cpu.registers.check_condition(cond) || !with_cond {
        cpu.pc = cpu.registers.pop_sp(&cpu.bus.borrow_mut());
    } else {
        cpu.pc = cpu.pc.wrapping_add(1);
    }
}

fn jp_imm16(cpu: &mut Cpu, instruction: u8, with_cond: bool) {
    let cond = if with_cond {
        utils::convert_index_to_cond(instruction)
    } else {
        Cond::None
    };

    let imm16 = utils::get_imm16(cpu);

    if cpu.registers.check_condition(cond) || !with_cond {
        cpu.pc = imm16;
    } else {
        cpu.pc = cpu.pc.wrapping_add(3);
    }
}

fn jp_hl(cpu: &mut Cpu) {
    let hl_value = cpu.registers.get_r16_value(R16::HL);
    cpu.pc = hl_value;
}

fn call_imm16(cpu: &mut Cpu, instruction: u8, with_cond: bool) {
    let cond = if with_cond {
        utils::convert_index_to_cond(instruction)
    } else {
        Cond::None
    };

    let imm16 = utils::get_imm16(cpu);

    if cpu.registers.check_condition(cond) || !with_cond {
        cpu.registers
            .push_sp(&mut cpu.bus.borrow_mut(), cpu.pc.wrapping_add(3));
        cpu.pc = imm16;
    } else {
        cpu.pc = cpu.pc.wrapping_add(3);
    }
}

fn rst_tgt3(cpu: &mut Cpu, instruction: u8) {
    let tgt3_index = (instruction & TGT3_MASK) >> 3;

    if (tgt3_index as usize) < RST_VEC.len() {
        let tgt3_address = RST_VEC[tgt3_index as usize];
        cpu.pc = cpu.registers.pop_sp(&cpu.bus.borrow_mut());
        cpu.pc = tgt3_address as u16;
    } else {
        panic!("Invalid tgt3_index: {}", tgt3_index);
    }
}

fn pop_r16(cpu: &mut Cpu, instruction: u8) {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.pop_sp(&cpu.bus.borrow_mut());
    cpu.registers.set_r16_value(r16, value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn pop_af(cpu: &mut Cpu) {
    let value = cpu.registers.pop_sp(&cpu.bus.borrow_mut());
    cpu.registers.set_af(value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn push_r16(cpu: &mut Cpu, instruction: u8) {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.get_r16_value(r16);
    cpu.registers.push_sp(&mut cpu.bus.borrow_mut(), value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn push_af(cpu: &mut Cpu) {
    let value = cpu.registers.get_af();
    cpu.registers.push_sp(&mut cpu.bus.borrow_mut(), value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn prefix(cpu: &mut Cpu) {
    let next_instruction = cpu.bus.borrow().read_byte(cpu.pc + 1);
    block_prefix::execute_instruction_block_prefix(cpu, next_instruction);
    // cpu.pc = cpu.pc.wrapping_add(1);
}

fn ldh_c_a(cpu: &mut Cpu) {
    let a_value = cpu.get_r8_value(R8::A);
    let c_value = cpu.get_r8_value(R8::C);

    let address = 0xFF00 + (c_value as u16);
    cpu.bus.borrow_mut().write_byte(address, a_value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn ldh_imm8_a(cpu: &mut Cpu) {
    let a_value = cpu.get_r8_value(R8::A);
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);

    let address = 0xFF00 + (imm8 as u16);
    cpu.bus.borrow_mut().write_byte(address, a_value);
    cpu.pc = cpu.pc.wrapping_add(2);
}

fn ld_imm16_a(cpu: &mut Cpu) {
    let a_value = cpu.get_r8_value(R8::A);
    let imm16 = utils::get_imm16(cpu);

    cpu.bus.borrow_mut().write_byte(imm16, a_value);
    cpu.pc = cpu.pc.wrapping_add(3);
}

fn ldh_a_c(cpu: &mut Cpu) {
    let c_value = cpu.get_r8_value(R8::C);
    let address = 0xFF00 + (c_value as u16);
    let value = cpu.bus.borrow().read_byte(address);

    cpu.set_r8_value(R8::A, value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

fn ldh_a_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1);
    let address = 0xFF00 + (imm8 as u16);
    let value = cpu.bus.borrow().read_byte(address);

    cpu.set_r8_value(R8::A, value);
    cpu.pc = cpu.pc.wrapping_add(2);
}

fn ld_a_imm16(cpu: &mut Cpu) {
    let imm16 = utils::get_imm16(cpu);
    let value = cpu.bus.borrow().read_byte(imm16);

    cpu.set_r8_value(R8::A, value);
    cpu.pc = cpu.pc.wrapping_add(3);
}

fn add_sp_imm8(cpu: &mut Cpu) {
    let offset = cpu.bus.borrow().read_byte(cpu.pc + 1) as i8;

    cpu.registers.add_sp_i8(offset);
    cpu.pc = cpu.pc.wrapping_add(2);
}

fn ld_hl_sp_add_imm8(cpu: &mut Cpu) {
    let imm8 = cpu.bus.borrow().read_byte(cpu.pc + 1) as i8;
    let sp = cpu.registers.get_sp();

    let result = sp.wrapping_add(imm8 as u16);
    cpu.registers.set_r16_value(R16::HL, result);

    cpu.registers.set_zero_flag(false);
    cpu.registers.set_subtract_flag(false);

    let sp_lo = sp & 0xFF;
    let imm_u8 = imm8 as u8;

    cpu.registers
        .set_half_carry_flag((sp_lo & 0xF) + (imm_u8 as u16 & 0xF) > 0xF);
    cpu.registers.set_carry_flag(sp_lo + imm_u8 as u16 > 0xFF);

    cpu.pc = cpu.pc.wrapping_add(2);
}

fn ld_sp_hl(cpu: &mut Cpu) {
    let hl_value = cpu.registers.get_r16_value(R16::HL);
    cpu.registers.set_sp(hl_value);
    cpu.pc = cpu.pc.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    #[test]
    fn test_add_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x10);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x20);
        execute_instruction_block3(&mut cpu, 0xC6); // ADD A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0x30);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_adc_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x10);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x20);
        cpu.registers.set_carry_flag(true);
        execute_instruction_block3(&mut cpu, 0xCE); // ADC A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0x31);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_sub_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x30);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x10);
        execute_instruction_block3(&mut cpu, 0xD6); // SUB A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0x20);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_sbc_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x30);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x10);
        cpu.registers.set_carry_flag(true);
        execute_instruction_block3(&mut cpu, 0xDE); // SBC A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0x1F);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_and_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0b1010);
        execute_instruction_block3(&mut cpu, 0xE6); // AND A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0b1000);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_xor_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0b1010);
        execute_instruction_block3(&mut cpu, 0xEE); // XOR A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0b0110);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_or_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1100);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0b1010);
        execute_instruction_block3(&mut cpu, 0xF6); // OR A, imm8

        assert_eq!(cpu.get_r8_value(R8::A), 0b1110);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_cp_a_imm8() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x20);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x20);
        execute_instruction_block3(&mut cpu, 0xFE); // CP A, imm8

        assert!(cpu.registers.get_zero_flag());
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_jp_imm16() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        execute_instruction_block3(&mut cpu, 0xC3); // JP imm16

        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_jp_cond_true() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        cpu.registers.set_zero_flag(true); // Condition Z = true
        execute_instruction_block3(&mut cpu, 0xCA); // JP Z, imm16

        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_jp_cond_false() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        cpu.registers.set_zero_flag(false); // Condition Z = false
        execute_instruction_block3(&mut cpu, 0xCA); // JP Z, imm16

        assert_eq!(cpu.pc, 0x0100 + 3); // Pas de saut
    }

    #[test]
    fn test_jp_hl() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0x1234);
        execute_instruction_block3(&mut cpu, 0xE9); // JP HL

        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_call_imm16() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        execute_instruction_block3(&mut cpu, 0xCD); // CALL imm16

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.registers.pop_sp(&cpu.bus.borrow_mut()), 0x0103);
    }

    #[test]
    fn test_call_cond_true() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        cpu.registers.set_carry_flag(true); // Condition C = true
        execute_instruction_block3(&mut cpu, 0xDC); // CALL C, imm16

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.registers.pop_sp(&cpu.bus.borrow_mut()), 0x0103);
    }

    #[test]
    fn test_call_cond_false() {
        let mut cpu = Cpu::default();
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.borrow_mut().write_byte(cpu.pc + 2, 0x12); // MSB
        cpu.registers.set_carry_flag(false); // Condition C = false
        execute_instruction_block3(&mut cpu, 0xDC); // CALL C, imm16

        assert_eq!(cpu.pc, 0x0100 + 3); // Pas de saut
    }

    #[test]
    fn test_rst_tgt3() {
        let mut cpu = Cpu::default();
        execute_instruction_block3(&mut cpu, 0xC7); // RST 0x00

        assert_eq!(cpu.pc, 0x0000);
    }

    #[test]
    fn test_pop_r16() {
        let mut cpu = Cpu::default();
        cpu.registers.push_sp(&mut cpu.bus.borrow_mut(), 0x1234);
        execute_instruction_block3(&mut cpu, 0xC1); // POP BC

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1234);
    }

    #[test]
    fn test_push_r16() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::DE, 0x5678);
        cpu.registers.push_sp(&mut cpu.bus.borrow_mut(), 0x0100);
        execute_instruction_block3(&mut cpu, 0xD5); // PUSH DE

        assert_eq!(cpu.registers.pop_sp(&cpu.bus.borrow_mut()), 0x5678);
    }

    #[test]
    fn test_add_sp_imm8_positive() {
        let mut cpu = Cpu::default();
        cpu.registers.set_sp(0xFFF0);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x10); // imm8 = +16
        execute_instruction_block3(&mut cpu, 0xE8); // ADD SP, imm8

        assert_eq!(cpu.registers.get_sp(), 0x0000); // Overflow
    }

    #[test]
    fn test_add_sp_imm8_negative() {
        let mut cpu = Cpu::default();
        cpu.registers.set_sp(0x0005);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0xFB); // imm8 = -5 (0xFB = -5 en i8)
        execute_instruction_block3(&mut cpu, 0xE8); // ADD SP, imm8

        assert_eq!(cpu.registers.get_sp(), 0x0000);
    }

    #[test]
    fn test_ld_hl_sp_add_imm8() {
        let mut cpu = Cpu::default();
        cpu.registers.set_sp(0xFFF0);
        cpu.bus.borrow_mut().write_byte(cpu.pc + 1, 0x10); // imm8 = +16
        execute_instruction_block3(&mut cpu, 0xF8); // LD HL, SP + imm8

        assert_eq!(cpu.registers.get_r16_value(R16::HL), 0x0000); // Overflow
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0x1234);
        execute_instruction_block3(&mut cpu, 0xF9); // LD SP, HL

        assert_eq!(cpu.registers.get_sp(), 0x1234);
    }

    // #[test]
    // fn test_di() {
    //     let mut cpu = Cpu::default();
    //     cpu.registers.set_interrupts_enabled(true);
    //     execute_instruction_block3(&mut cpu, 0xF3); // DI

    //     assert!(!cpu.registers.get_interrupts_enabled());
    // }

    // #[test]
    // fn test_ei() {
    //     let mut cpu = Cpu::default();
    //     cpu.registers.set_interrupts_enabled(false);
    //     execute_instruction_block3(&mut cpu, 0xFB); // EI

    //     assert!(cpu.registers.get_interrupts_enabled());
    // }

    #[test]
    fn test_ldh_a_c() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::C, 0x10);
        cpu.bus.borrow_mut().write_byte(0xFF10, 0x42);
        execute_instruction_block3(&mut cpu, 0xF2); // LDH A, [C]

        assert_eq!(cpu.get_r8_value(R8::A), 0x42);
        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_pop_af() {
        let mut cpu = Cpu::default();

        // Pousse une valeur sur la pile
        cpu.registers.push_sp(&mut cpu.bus.borrow_mut(), 0x1234);

        // Exécute l'instruction POP AF
        execute_instruction_block3(&mut cpu, 0xF1); // POP AF

        // Vérifie que le registre AF contient la valeur extraite de la pile
        assert_eq!(cpu.registers.get_af(), 0x1234);

        // Vérifie que le compteur de programme a été incrémenté correctement
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_rst_38h() {
        let mut cpu = Cpu::default();
        execute_instruction_block3(&mut cpu, 0xFF); // RST 38h

        assert_eq!(cpu.pc, 0x0038);
    }
}
