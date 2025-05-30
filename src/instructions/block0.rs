use crate::cpu::CPU;
use crate::registers::{R16, R8};

const R16_MASK: u8 = 0b00110000;
const R8_MASK: u8 = 0b00111000;
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

	match_opcode.retain(|&opcode| (instruction & mask4) == (opcode & mask4));
	if match_opcode.len() > 1 {
		match_opcode.retain(|&opcode| (instruction & mask_start_3) == (opcode & mask_start_3));
	}
	
	if match_opcode.len() == 1 {
		match_opcode[0]
	} else {
		panic!("No unique instruction found for opcode: {:#04x}", instruction);
	}
}

pub fn match_instruction_block0(cpu: &mut CPU, instruction: u8) {
	let opcode = get_instruction_block0(instruction);

	match opcode {
		0b00000000 => { cpu.pc += 1; }, // nop
		0b00000001 => load_r16_imm16(cpu, instruction),
		0b00000010 => load_r16mem_a(cpu, instruction),
		0b00001010 => load_a_r16mem(cpu, instruction),
		//TODO: Implement ld [imm16], sp
		0b00000011 => inc_r16(cpu, instruction),
		0b00001011 => dec_r16(cpu, instruction),
		0b00001001 => add_hl_r16(cpu, instruction),
		0b00000100 => inc_r8(cpu, instruction),
		0b00000101 => dec_r8(cpu, instruction),
		0b00000110 => ld_r8_imm8(cpu, instruction),
		0b00000111 => rlca(cpu),
		0b00001111 => rrca(cpu),
		_ => panic!("Unknown opcode: {:#04x}", opcode),
	}
}

fn convert_index_to_r16(instruction: u8) -> R16 {
	let r16_index = (instruction & R16_MASK) >> 4;
	R16::from(r16_index)
}

fn convert_index_to_r8(instruction: u8) -> R8 {
	let r8_index: u8 = (instruction & R8_MASK) >> 3;
	R8::from(r8_index)
}

fn load_r16_imm16(cpu: &mut CPU, instruction: u8) {
	let lsb = cpu.bus.read_byte(cpu.pc + 1) as u16;
	let msb = cpu.bus.read_byte(cpu.pc + 2) as u16;
	let imm16 = (msb << 8) | lsb;
	let r16 = R16::from((instruction & R16_MASK) >> 4);

	cpu.registers.set_r16_value(r16, imm16);
	cpu.pc += 3;
}

fn load_r16mem_a(cpu: &mut CPU, instruction: u8) {
	let r16 = convert_index_to_r16(instruction);
	let a_value = cpu.registers.get_a();

	cpu.registers.set_r16_mem_value(&mut cpu.bus, r16, a_value);
	cpu.pc += 1;
}

fn load_a_r16mem(cpu: &mut CPU, instruction: u8) {
	let r16 = convert_index_to_r16(instruction);
	let value = cpu.registers.get_r16_mem_value(&cpu.bus, r16);

	cpu.registers.set_r8_value(R8::A, value);
	cpu.pc += 1;
}

fn inc_r16(cpu: &mut CPU, instruction: u8) {
	let r16 = convert_index_to_r16(instruction);
	let value = cpu.registers.get_r16_value(r16);

	cpu.registers.set_r16_value(r16, value.wrapping_add(1));
	cpu.pc += 1;
}

fn dec_r16(cpu: &mut CPU, instruction: u8) {
	let r16 = convert_index_to_r16(instruction);
	let value = cpu.registers.get_r16_value(r16);

	cpu.registers.set_r16_value(r16, value.wrapping_sub(1));
	cpu.pc += 1;
}

fn add_hl_r16(cpu: &mut CPU, instruction: u8) {
	let r16 = convert_index_to_r16(instruction);
	let value = cpu.registers.get_r16_value(r16);
	cpu.registers.add_to_r16(R16::HL, value);
}

fn inc_r8(cpu: &mut CPU, instruction: u8) {
	let r8 = convert_index_to_r8(instruction);
	let value = cpu.registers.get_r8_value(r8);
	let new_value = value.wrapping_add(1);

	cpu.registers.set_zero_flag(new_value == 0);
	cpu.registers.set_subtract_flag(false);
	cpu.registers.set_half_carry_flag((new_value & 0x0F) + 1 > 0x0F);

	cpu.registers.set_r8_value(r8, new_value);
	cpu.pc += 1;
}

fn dec_r8(cpu: &mut CPU, instruction: u8) {
	let r8 = convert_index_to_r8(instruction);
	let value = cpu.registers.get_r8_value(r8);
	let new_value = value.wrapping_sub(1);

	cpu.registers.set_zero_flag(new_value == 0);
	cpu.registers.set_subtract_flag(true);
	cpu.registers.set_half_carry_flag((new_value & 0x0F) + 1 > 0x0F);
	cpu.registers.set_r8_value(r8, new_value);
	cpu.pc += 1;
}

fn ld_r8_imm8(cpu: &mut CPU, instruction: u8) {
	let imm8 = cpu.bus.read_byte(cpu.pc + 1);
	let r8 = convert_index_to_r8(instruction);

	cpu.registers.set_r8_value(r8, imm8);
	cpu.pc += 2;
}

fn rlca(cpu: &mut CPU) {
	cpu.registers.rotate_left(R8::A,true);
	cpu.pc += 1;
}

fn rrca(cpu: &mut CPU) {
	cpu.registers.rotate_right(R8::A, true);
	cpu.pc += 1;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::CPU;
    use crate::registers::Registers;


    #[test]
    fn test_nop() {
        let mut cpu = CPU::default();
        match_instruction_block0(&mut cpu, 0x00); // NOP
        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn test_ld_r16_imm16_bc() {
        let mut cpu = CPU::default();
        cpu.bus.write_byte(1, 0x34); // LSB
        cpu.bus.write_byte(2, 0x12); // MSB
        match_instruction_block0(&mut cpu, 0x01); // LD BC, 0x1234

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1234);
        assert_eq!(cpu.pc, 3);
    }

    #[test]
    fn test_ld_r16mem_a() {
        let mut cpu = CPU::default();
        cpu.registers.set_r16_value(R16::DE, 0xC000);
        cpu.registers.set_r8_value(R8::A, 0x42);
        match_instruction_block0(&mut cpu, 0x12); // LD [DE], A

        assert_eq!(cpu.bus.read_byte(0xC000), 0x42);
    }

    #[test]
    fn test_ld_a_r16mem() {
        let mut cpu = CPU::default();
        cpu.registers.set_r16_value(R16::DE, 0xC000);
        cpu.bus.write_byte(0xC000, 0xAB);
        match_instruction_block0(&mut cpu, 0x1A); // LD A, [DE]

        assert_eq!(cpu.registers.get_a(), 0xAB);
    }

    #[test]
    fn test_inc_r16() {
        let mut cpu = CPU::default();
        cpu.registers.set_r16_value(R16::BC, 0x1234);
        match_instruction_block0(&mut cpu, 0x03); // INC BC

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1235);
    }

    #[test]
    fn test_dec_r16() {
        let mut cpu = CPU::default();
        cpu.registers.set_r16_value(R16::BC, 0x1234);
        match_instruction_block0(&mut cpu, 0x0B); // DEC BC

        assert_eq!(cpu.registers.get_r16_value(R16::BC), 0x1233);
    }

    #[test]
    #[should_panic]
    fn test_invalid_instruction_panics() {
        let mut cpu = CPU::default();
        match_instruction_block0(&mut cpu, 0xFF);
    }
}
