use crate::cpu::CPU;

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
		return match_opcode[0];
	} else {
		panic!("No unique instruction found for opcode: {:#04x}", instruction);
	}
}

pub fn match_instruction_block0(cpu: &mut CPU, instruction: u8) {
	let r16_mask = 0b00110000;
	// let cond_mask = 0b00011000;
	let opcode = get_instruction_block0(instruction);

	match opcode {
		0b00000000 => { cpu.pc += 1; }, // nop
		0b00000001 => load_r16_imm16(cpu, instruction),
		0b00000010 => load_r16mem_a(cpu, instruction),
		0b00001010 => load_a_r16mem(cpu, instruction),
		//TODO: Implement ld [imm16], sp
		_ => panic!("Unknown opcode: {:#04x}", opcode),
	}
}

fn load_r16_imm16(cpu: &mut CPU, instruction: u8) {
	let lsb = cpu.bus.read_byte(cpu.pc + 1) as u16;
	let msb = cpu.bus.read_byte(cpu.pc + 2) as u16;
	let imm16 = (msb << 8) | lsb;
	cpu.load_r16(imm16, (instruction & 0b00110000) >> 4);
	cpu.pc += 2; // Increment PC by 2 to skip the immediate value bytes
}

fn load_r16mem_a(cpu: &mut CPU, instruction: u8) {
	let r16_index = (instruction & 0b00110000) >> 4;
	let a_value = cpu.registers.get_a();
	cpu.registers.set_r16_mem_value(&mut cpu.bus, r16_index, a_value);
	cpu.pc += 1; // Increment PC by 1 to skip the instruction byte
}

fn load_a_r16mem(cpu: &mut CPU, instruction: u8) {
	let r16_index = (instruction & 0b00110000) >> 4;
	let value = cpu.registers.get_r16_mem_value(&cpu.bus, r16_index);
	cpu.load_r8(value, 7);
	cpu.pc += 1; // Increment PC by 1 to skip the instruction byte
}

fn inc_r16(cpu: &mut CPU, instruction: u8) {
	let r16_index = (instruction & 0b00110000) >> 4;
	let value = cpu.registers.get_r16_value(r16_index);
	cpu.registers.set_r16_value(r16_index, value.wrapping_add(1));
	cpu.pc += 1; // Increment PC by 1 to skip the instruction byte
}
