pub mod block0;

use crate::cpu::CPU;

pub fn execute_instruction(cpu: &mut CPU, instruction: u8) {
	let block_mask = 0b11000000;
	let block = (instruction & block_mask) >> 6;
	match block {
		0b00 => block0::match_instruction_block0(cpu, instruction),
		// Add more blocks here as needed
		_ => panic!("Unknown instruction block: {}", block),
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ld_r16_imm16_for_all_registers() {
        let test_cases = vec![
            (0x01, "BC"), // LD BC, imm16
            (0x11, "DE"), // LD DE, imm16
            (0x21, "HL"), // LD HL, imm16
        ];

        for (opcode, reg_name) in test_cases {
            let mut cpu = CPU::default();
            cpu.pc = 0x0000;

            // Valeur immédiate à charger : 0xABCD
            let lsb = 0xCD;
            let msb = 0xAB;
            let expected = 0xABCD;

            // Écrit l'instruction et les imm16 suivants en mémoire
            cpu.bus.write_byte(0x0000, opcode);
            cpu.bus.write_byte(0x0001, lsb);
            cpu.bus.write_byte(0x0002, msb);

            // Appelle la fonction
            execute_instruction(&mut cpu, opcode);

            // Vérifie que la bonne valeur est dans le bon registre
            let value = match reg_name {
                "BC" => cpu.registers.get_bc(),
                "DE" => cpu.registers.get_de(),
                "HL" => cpu.registers.get_hl(),
                _ => panic!("Unknown register name: {}", reg_name),
            };

            assert_eq!(
                value, expected,
                "Expected {} to be 0x{:04X}, got 0x{:04X}",
                reg_name, expected, value
            );
        }
    }
}

