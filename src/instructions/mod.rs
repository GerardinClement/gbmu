pub mod block0;
pub mod conditions;

use crate::cpu::cpu::CPU;

pub fn execute_instruction(cpu: &mut CPU, instruction: u8) {
	let block_mask = 0b11000000;
	let block = (instruction & block_mask) >> 6;
	match block {
		0b00 => block0::match_instruction_block0(cpu, instruction),
		// Add more blocks here as needed
		_ => panic!("Unknown instruction block: {}", block),
	}
}

