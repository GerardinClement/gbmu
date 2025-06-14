#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cpu::Cpu;
use crate::cpu::conditions::Cond;
use crate::cpu::registers::{R8, R16, R16Mem};
use crate::cpu::utils;

const COND_MASK: u8 = 0b00011000;

const INSTRUCTIONS_BLOCK0: [u8; 22] = [
    0b00000000, //nop
    0b00000001, //ld r16, imm16
    0b00000010, //ld [r16mem], a
    0b00001010, //ld a, [r16mem]
    0b00001000, //ld [imm16], sp
    0b00000011, //inc r16
    0b00001011, //dec r16
    0b00001001, //add hl, r16
    0b00000100, //inc r8
    0b00000101, //dec r8
    0b00000110, //ld r8, imm8
    0b00000111, //rlca
    0b00001111, //rrca
    0b00010111, //rla
    0b00011111, //rra
    0b00100111, //daa
    0b00101111, //cpl
    0b00110111, //scf
    0b00111111, //ccf
    0b00011000, //jr imm8
    0b00100000, //jr cond, imm8
    0b00010000, //stop
];

/// GET the instruction based on the opcode and returns the corresponding instruction.
fn get_instruction_block0(instruction: u8) -> u8 {
    let mask3 = 0b00000111;
    let mask4 = 0b00001111;
    let mask_start_3 = 0b11100000;

    if INSTRUCTIONS_BLOCK0.contains(&instruction) {
        return instruction;
    }

    let mut match_opcode: Vec<u8> = INSTRUCTIONS_BLOCK0
        .iter()
        .cloned()
        .filter(|&opcode| (instruction & mask3) == (opcode & mask3))
        .collect();

    if match_opcode.len() == 1 {
        return match_opcode[0];
    }

    let mut match_opcode_cpy = match_opcode.clone();

    match_opcode.retain(|&opcode| (instruction & mask4) == (opcode & mask4));
    if match_opcode.len() > 1 {
        match_opcode_cpy.retain(|&opcode| (instruction & mask_start_3) == (opcode & mask_start_3));
        if match_opcode_cpy.len() == 1 {
            return match_opcode_cpy[0];
        }
    }

    if match_opcode.len() == 1 {
        match_opcode[0]
    } else {
        panic!(
            "No unique instruction found for opcode: {:#04x}",
            instruction
        );
    }
}

pub fn execute_instruction_block0(cpu: &mut Cpu, instruction: u8) {
    let opcode = get_instruction_block0(instruction);

    match opcode {
        0b00000000 => {
            cpu.pc += 1;
        }
        0b00000001 => load_r16_imm16(cpu, instruction),
        0b00000010 => load_r16mem_a(cpu, instruction),
        0b00001010 => load_a_r16mem(cpu, instruction),
        0b00001000 => load_mem_imm16_sp(cpu),
        0b00000011 => inc_r16(cpu, instruction),
        0b00001011 => dec_r16(cpu, instruction),
        0b00001001 => add_hl_r16(cpu, instruction),
        0b00000100 => inc_r8(cpu, instruction),
        0b00000101 => dec_r8(cpu, instruction),
        0b00000110 => ld_r8_imm8(cpu, instruction),
        0b00000111 => rotate_left(cpu, true),
        0b00001111 => rotate_right(cpu, true),
        0b00010111 => rotate_left(cpu, false),
        0b00011111 => rotate_right(cpu, false),
        0b00100111 => daa(cpu),
        0b00101111 => cpl(cpu),
        0b00110111 => scf(cpu),
        0b00111111 => ccf(cpu),
        0b00011000 => jr(cpu, instruction, false),
        0b00100000 => jr(cpu, instruction, true),
        //implement STOP
        _ => cpu.pc = cpu.pc.wrapping_add(1),
    }
}

fn convert_index_to_cond(instruction: u8) -> Cond {
    let cond_index = (instruction & COND_MASK) >> 3;
    Cond::from(cond_index)
}

fn load_r16_imm16(cpu: &mut Cpu, instruction: u8) {
    let imm16 = utils::get_imm16(cpu);
    let r16 = R16::from((instruction & utils::R16_MASK) >> 4);

    cpu.registers.set_r16_value(r16, imm16);
    cpu.pc += 3;
}

fn load_r16mem_a(cpu: &mut Cpu, instruction: u8) {
    let r16_mem = utils::convert_index_to_r16_mem(instruction);
    let a_value = cpu.registers.get_a();

    cpu.registers
        .set_r16_mem_value(&mut cpu.bus, R16::from(r16_mem), a_value);
    if r16_mem == R16Mem::HLincrement || r16_mem == R16Mem::HLdecrement {
        utils::modify_hl(cpu, r16_mem);
    }
    cpu.pc += 1;
}

fn load_a_r16mem(cpu: &mut Cpu, instruction: u8) {
    let r16_mem = utils::convert_index_to_r16_mem(instruction);
    let value = cpu
        .registers
        .get_r16_mem_value(&cpu.bus, R16::from(r16_mem));

    cpu.set_r8_value(R8::A, value);
    if r16_mem == R16Mem::HLincrement || r16_mem == R16Mem::HLdecrement {
        utils::modify_hl(cpu, r16_mem);
    }

    cpu.pc += 1;
}

fn load_mem_imm16_sp(cpu: &mut Cpu) {
    let sp_msb = (cpu.registers.get_sp() >> 8) as u8;
    let sp_lsb = (cpu.registers.get_sp() & 0xFF) as u8;

    let imm16 = utils::get_imm16(cpu);

    cpu.bus.write_byte(imm16, sp_lsb);
    cpu.bus.write_byte(imm16 + 1, sp_msb);

    cpu.pc = cpu.pc.wrapping_add(3);
}

fn inc_r16(cpu: &mut Cpu, instruction: u8) {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.get_r16_value(r16);

    cpu.registers.set_r16_value(r16, value.wrapping_add(1));
    cpu.pc += 1;
}

fn dec_r16(cpu: &mut Cpu, instruction: u8) {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.get_r16_value(r16);

    cpu.registers.set_r16_value(r16, value.wrapping_sub(1));
    cpu.pc += 1;
}

fn add_hl_r16(cpu: &mut Cpu, instruction: u8) {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.get_r16_value(r16);
    cpu.registers.add_to_r16(R16::HL, value);

    cpu.pc = cpu.pc.wrapping_add(1);
}

fn inc_r8(cpu: &mut Cpu, instruction: u8) {
    let r8 = utils::convert_dest_index_to_r8(instruction);
    let value = cpu.get_r8_value(r8);
    let new_value = value.wrapping_add(1);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag((value & 0x0F) + 1 > 0x0F);

    cpu.set_r8_value(r8, new_value);
    cpu.pc += 1;
}

fn dec_r8(cpu: &mut Cpu, instruction: u8) {
    let r8 = utils::convert_dest_index_to_r8(instruction);
    let value = cpu.get_r8_value(r8);
    let new_value = value.wrapping_sub(1);

    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(true);
    cpu.registers.set_half_carry_flag((value & 0x0F) == 0x00);
    cpu.set_r8_value(r8, new_value);
    cpu.pc += 1;
}

fn ld_r8_imm8(cpu: &mut Cpu, instruction: u8) {
    let imm8 = cpu.bus.read_byte(cpu.pc + 1);
    let r8 = utils::convert_dest_index_to_r8(instruction);

    cpu.set_r8_value(r8, imm8);
    cpu.pc += 2;
}

fn rotate_left(cpu: &mut Cpu, carry: bool) {
    cpu.registers.rotate_left(R8::A, carry);
    cpu.pc += 1;
}

fn rotate_right(cpu: &mut Cpu, carry: bool) {
    cpu.registers.rotate_right(R8::A, carry);
    cpu.pc += 1;
}

fn daa(cpu: &mut Cpu) {
    let mut adjust: u8 = 0;
    let mut a = cpu.registers.get_a();
    if cpu.registers.get_subtract_flag() {
        if cpu.registers.get_half_carry_flag() {
            adjust += 0x6;
        }
        if cpu.registers.get_carry_flag() {
            adjust += 0x60;
        }
        a = a.wrapping_sub(adjust);
        cpu.set_r8_value(R8::A, a);
    } else {
        if cpu.registers.get_half_carry_flag() || (a & 0xF) > 0x9 {
            adjust += 0x6;
        }
        if cpu.registers.get_carry_flag() || a > 0x99 {
            adjust += 0x60;
            cpu.registers.set_carry_flag(true);
        }
        a = a.wrapping_add(adjust);
        cpu.set_r8_value(R8::A, a);
    }
    cpu.registers.set_zero_flag(a == 0);
    cpu.registers.set_half_carry_flag(false);
    cpu.pc += 1;
}

fn cpl(cpu: &mut Cpu) {
    let a = cpu.get_r8_value(R8::A);
    let new_value = !a;
    cpu.set_r8_value(R8::A, new_value);
    cpu.registers.set_subtract_flag(true);
    cpu.registers.set_half_carry_flag(true);
    cpu.pc += 1;
}

fn scf(cpu: &mut Cpu) {
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(true);
    cpu.pc += 1;
}

fn ccf(cpu: &mut Cpu) {
    let carry_value = cpu.registers.get_carry_flag();
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(false);
    cpu.registers.set_carry_flag(!carry_value);
    cpu.pc += 1;
}

fn jr(cpu: &mut Cpu, instruction: u8, has_cond: bool) {
    if has_cond {
        let cond = convert_index_to_cond(instruction);
        if !cond.test(&mut cpu.registers) {
            cpu.pc = cpu.pc.wrapping_add(2);
            return;
        }
    }
    let offset = cpu.bus.read_byte(cpu.pc + 1) as i8;
    cpu.pc = cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;

    #[test]
    fn test_nop() {
        let mut cpu = Cpu::default();
        execute_instruction_block0(&mut cpu, 0x00); // NOP
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_ld_r16_imm16_bc() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(cpu.pc + 1, 0x34); // LSB
        cpu.bus.write_byte(cpu.pc + 2, 0x12); // MSB
        execute_instruction_block0(&mut cpu, 0x01); // LD BC, 0x1234

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1234);
        assert_eq!(cpu.pc, 0x0100 + 3);
    }

    #[test]
    fn test_ld_r16mem_a() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::DE, 0xC000);
        cpu.set_r8_value(R8::A, 0x42);
        execute_instruction_block0(&mut cpu, 0x12); // LD [DE], A

        assert_eq!(cpu.bus.read_byte(0xC000), 0x42);
    }

    #[test]
    fn test_ld_a_r16mem() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::DE, 0xC000);
        cpu.bus.write_byte(0xC000, 0xAB);
        execute_instruction_block0(&mut cpu, 0x1A); // LD A, [DE]

        assert_eq!(cpu.registers.get_a(), 0xAB);
    }

    #[test]
    fn test_ld_mem_imm16_sp() {
        let mut cpu = Cpu::default();

        // Simuler l'instruction en mémoire : opcode = 0x08, suivi de l'adresse imm16 (par ex. 0x1234)
        cpu.bus.write_byte(cpu.pc, 0x08); // opcode LD (n16), SP
        cpu.bus.write_byte(cpu.pc + 1, 0x34); // low byte de l’adresse
        cpu.bus.write_byte(cpu.pc + 2, 0x12); // high byte de l’adresse

        cpu.registers.set_sp(0xC123);

        // Exécuter l'instruction
        execute_instruction_block0(&mut cpu, 0x08);

        // Vérifier que SP a bien été écrit à l'adresse 0x1234
        assert_eq!(cpu.bus.read_byte(0x1234), 0x23); // low byte
        assert_eq!(cpu.bus.read_byte(0x1235), 0xC1); // high byte
    }

    #[test]
    fn test_inc_indirect_hl() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0xC000);
        cpu.bus.write_byte(0xC000, 0x3F);

        execute_instruction_block0(&mut cpu, 0x34); // INC [HL]

        let result = cpu.bus.read_byte(0xC000);
        assert_eq!(result, 0x40);
        assert!(!cpu.registers.get_zero_flag());
        assert!(!cpu.registers.get_subtract_flag());
        assert!(cpu.registers.get_half_carry_flag()); // 0x3F -> 0x40 déclenche un half carry
    }

    #[test]
    fn test_inc_r16() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::BC, 0x1234);
        execute_instruction_block0(&mut cpu, 0x03); // INC BC

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1235);
    }

    #[test]
    fn test_dec_r16() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::BC, 0x1234);
        execute_instruction_block0(&mut cpu, 0x0B); // DEC BC

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1233);
    }

    #[test]
    #[should_panic]
    fn test_invalid_instruction_panics() {
        let mut cpu = Cpu::default();
        execute_instruction_block0(&mut cpu, 0xFF);
    }

    #[test]
    fn test_rlca() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b1001_0001); // A = 0x91
        cpu.registers.set_carry_flag(false);

        execute_instruction_block0(&mut cpu, 0x07); // RLCA

        assert_eq!(cpu.registers.get_a(), 0b0010_0011); // rotation gauche
        assert!(cpu.registers.get_carry_flag()); // bit 7 = 1
        assert!(!cpu.registers.get_zero_flag()); // toujours false
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_rrca() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b0000_0001); // A = 0x01
        cpu.registers.set_carry_flag(false);

        execute_instruction_block0(&mut cpu, 0x0F); // RRCA

        assert_eq!(cpu.registers.get_a(), 0b1000_0000); // rotation droite
        assert!(cpu.registers.get_carry_flag()); // bit 7 = 1
        assert!(!cpu.registers.get_zero_flag()); // toujours false
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_rla() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b0101_0101); // A = 0x55
        cpu.registers.set_carry_flag(true); // carry = 1

        execute_instruction_block0(&mut cpu, 0x17); // RLA

        assert_eq!(cpu.registers.get_a(), 0b1010_1011);
        assert!(!cpu.registers.get_carry_flag()); // bit 7 = 0
        assert!(!cpu.registers.get_zero_flag());
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_rra() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0b0000_0000); // A = 0x00
        cpu.registers.set_carry_flag(true); // carry = 1

        execute_instruction_block0(&mut cpu, 0x1F); // RRA

        assert_eq!(cpu.registers.get_a(), 0b1000_0000); // carry passé en bit 7
        assert!(!cpu.registers.get_carry_flag()); // bit 0 = 0
        assert!(!cpu.registers.get_zero_flag());
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_daa_addition_no_carry() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x09);
        cpu.registers.set_subtract_flag(false); // addition
        cpu.registers.set_half_carry_flag(true); // A & 0xF > 9 → BCD adjust
        cpu.registers.set_carry_flag(false);

        execute_instruction_block0(&mut cpu, 0x27); // DAA

        assert_eq!(cpu.get_r8_value(R8::A), 0xF); // 0x09 + 0x06
        assert!(!cpu.registers.get_zero_flag());
        assert!(!cpu.registers.get_half_carry_flag()); // cleared
        assert!(!cpu.registers.get_carry_flag());
        assert_eq!(cpu.pc, 0x0100 + 1);
    }

    #[test]
    fn test_daa_addition_with_carry() {
        let mut cpu = Cpu::default();
        cpu.set_r8_value(R8::A, 0x9A); // A invalide en BCD
        cpu.registers.set_subtract_flag(false); // addition
        cpu.registers.set_half_carry_flag(false);
        cpu.registers.set_carry_flag(false);

        execute_instruction_block0(&mut cpu, 0x27); // DAA

        assert_eq!(cpu.get_r8_value(R8::A), 0xDC); // 0x9A + 0x66 = 0x100 → overflow
        assert!(!cpu.registers.get_zero_flag());
        assert!(cpu.registers.get_carry_flag()); // overflow → carry
        assert!(!cpu.registers.get_half_carry_flag());
    }

    #[test]
    fn test_jr_no_condition_positive_offset() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(0x0101, 0x05); // offset = +5

        jr(&mut cpu, 0x18, false); // JR unconditional

        assert_eq!(cpu.pc, 0x0100 + 2 + 5);
    }

    #[test]
    fn test_jr_no_condition_negative_offset() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(0x0101, 0xFB); // offset = -5 (0xFB = -5 en i8)

        jr(&mut cpu, 0x18, false); // JR unconditional

        assert_eq!(cpu.pc, 0x0100 + 2 - 5);
    }

    #[test]
    fn test_jr_condition_true() {
        let mut cpu = Cpu::default();

        cpu.bus.write_byte(0x0101, 0x02); // offset = +2
        cpu.registers.set_zero_flag(true); // Z = 1

        jr(&mut cpu, 0x28, true); // JR Z, +2 (opcode 0x28)

        assert_eq!(cpu.pc, 0x0100 + 2 + 2);
    }

    #[test]
    fn test_jr_condition_false() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(0x0101, 0x05); // offset = +5
        cpu.registers.set_zero_flag(false); // Z = 0

        jr(&mut cpu, 0x28, true); // JR Z, +5 (opcode 0x28), mais Z = 0 → saute pas

        assert_eq!(cpu.pc, 0x0100); // Pas de saut
    }

    #[test]
    fn test_jr_condition_carry() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(0x0101, 0x03); // offset = +3
        cpu.registers.set_carry_flag(true);

        jr(&mut cpu, 0x38, true); // JR C, +3

        assert_eq!(cpu.pc, 0x0100 + 2 + 3);
    }

    #[test]
    fn test_jr_condition_not_carry() {
        let mut cpu = Cpu::default();
        cpu.bus.write_byte(0x0101, 0x03); // offset = +3
        cpu.registers.set_carry_flag(false);

        jr(&mut cpu, 0x38, true); // JR C, +3 → condition fausse

        assert_eq!(cpu.pc, 0x0100 + 2);
    }

    #[test]
    fn test_load_a_r16mem_hl_increment() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0xC000);
        cpu.bus.write_byte(0xC000, 0x42);
        execute_instruction_block0(&mut cpu, 0x2A); // LD A, [HL+]

        assert_eq!(cpu.registers.get_a(), 0x42);
        assert_eq!(cpu.registers.get_r16_value(R16::HL), 0xC001); // HL incremented
    }

    #[test]
    fn test_load_a_r16mem_hl_decrement() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0xC001);
        cpu.bus.write_byte(0xC001, 0x42);
        execute_instruction_block0(&mut cpu, 0x3A); // LD A, [HL-]

        assert_eq!(cpu.registers.get_a(), 0x42);
        assert_eq!(cpu.registers.get_r16_value(R16::HL), 0xC000); // HL decremented
    }

    #[test]
    fn test_load_a_r16mem_standard() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::DE, 0xC000);
        cpu.bus.write_byte(0xC000, 0x42);
        execute_instruction_block0(&mut cpu, 0x1A); // LD A, [DE]

        assert_eq!(cpu.registers.get_a(), 0x42);
        assert_eq!(cpu.registers.get_r16_value(R16::DE), 0xC000); // DE unchanged
    }

    #[test]
    fn test_load_a_r16mem_hl_boundary_increment() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0xFFFF);
        cpu.bus.write_byte(0xFFFF, 0x42);
        execute_instruction_block0(&mut cpu, 0x2A); // LD A, [HL+]

        assert_eq!(cpu.registers.get_a(), 0x42);
        assert_eq!(cpu.registers.get_r16_value(R16::HL), 0x0000); // HL wraps around
    }

    #[test]
    fn test_load_a_r16mem_hl_boundary_decrement() {
        let mut cpu = Cpu::default();
        cpu.registers.set_r16_value(R16::HL, 0x0000);
        cpu.bus.write_byte(0x0000, 0x42);
        execute_instruction_block0(&mut cpu, 0x3A); // LD A, [HL-]

        assert_eq!(cpu.registers.get_a(), 0x42);
        assert_eq!(cpu.registers.get_r16_value(R16::HL), 0xFFFF); // HL wraps around
    }

    // #[test]
    // fn test_jr_nz_condition_true() {
    //     let mut cpu = Cpu::default();

    //     // Simuler l'instruction en mémoire : opcode = 0x28, suivi de l'offset imm8
    //     cpu.bus.write_byte(cpu.pc + 1, 0x05); // offset = +5
    //     cpu.registers.set_zero_flag(false); // NZ (Not Zero) condition est vraie

    //     execute_instruction_block0(&mut cpu, 0x28); // JR NZ, +5

    //     // Vérifier que le saut a été effectué
    //     assert_eq!(cpu.pc, 0x0100 + 2 + 5);
    // }

    // #[test]
    // fn test_jr_nz_condition_false() {
    //     let mut cpu = Cpu::default();

    //     // Simuler l'instruction en mémoire : opcode = 0x28, suivi de l'offset imm8
    //     cpu.bus.write_byte(cpu.pc + 1, 0x05); // offset = +5
    //     cpu.registers.set_zero_flag(true); // NZ (Not Zero) condition est fausse

    //     execute_instruction_block0(&mut cpu, 0x28); // JR NZ, +5

    //     // Vérifier que le saut n'a pas été effectué
    //     assert_eq!(cpu.pc, 0x0100 + 2); // Pas de saut, seulement l'instruction consommée
    // }
}
