#![allow(unused_variables)]
#![allow(dead_code)]

pub mod block0;
pub mod block1;
pub mod block2;
pub mod block3;
pub mod block_prefix;
pub mod conditions;
pub mod flags_registers;
pub mod registers;
pub mod utils;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::cpu::registers::{R8, R16, Registers};
use crate::mmu::Mmu;

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub bus: Rc<RefCell<Mmu>>, // TODO refacto each mention of 'bus' as 'mmu'
    pub ime: bool,
    pub ime_delay: bool, // mimic hardware delay in EI
}

impl Cpu {
    pub fn new(bus: Rc<RefCell<Mmu>>) -> Self {
        Cpu {
            registers: Registers::default(),
            bus,
            pc: 0x0100,
            ime: false,
            ime_delay: false,
        }
    }

    fn execute_instruction(&mut self, instruction: u8) {
        let block_mask = 0b11000000;
        let block = (instruction & block_mask) >> 6;
        match block {
            0b00 => block0::execute_instruction_block0(self, instruction),
            0b01 => block1::execute_instruction_block1(self, instruction),
            0b10 => block2::execute_instruction_block2(self, instruction),
            0b11 => block3::execute_instruction_block3(self, instruction),
            _ => {
                println!("Unknown instruction block: {}", block);
                self.pc = self.pc.wrapping_add(1);
            }
        }
    }

    pub fn step(&mut self) {
        if self.ime {
            if let Some(interrupt) = self.bus.borrow().interrupts_next_request() {
                self.ime = false;
                self.bus.borrow_mut().interrupts_clear_request(interrupt);

                let ret_addr = self.pc;

                let sp1 = self.registers.get_sp().wrapping_sub(1);
                self.registers.set_sp(sp1);
                self.bus.borrow_mut().write_byte(sp1, (ret_addr >> 8) as u8);

                let sp2 = sp1.wrapping_sub(1);
                self.registers.set_sp(sp2);
                self.bus.borrow_mut().write_byte(sp2, (ret_addr & 0xFF) as u8);

                // Jump to the interrupt vector
                self.pc = interrupt.vector();
                // Stop further execution this cycle
                return;
            }
        }
        let instruction_byte = self.bus.borrow().read_byte(self.pc);
        // println!("pc: 0x{:02X}", self.pc);
        // println!("opcode: 0x{:02X}", instruction_byte);
        self.execute_instruction(instruction_byte);

        if self.ime_delay {
            self.ime = true;
            self.ime_delay = false;
        }

        println!("{}", self);
    }

    pub fn get_r8_value(&self, register: R8) -> u8 {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.borrow().read_byte(addr)
            }
            _ => self.registers.get_r8_value(register),
        }
    }

    pub fn set_r8_value(&mut self, register: R8, value: u8) {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.borrow_mut().write_byte(addr, value);
            }
            _ => self.registers.set_r8_value(register, value),
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.get_r8_value(R8::A),
            self.registers.get_flags_u8(),
            self.registers.get_r8_value(R8::B),
            self.registers.get_r8_value(R8::C),
            self.registers.get_r8_value(R8::D),
            self.registers.get_r8_value(R8::E),
            self.registers.get_r8_value(R8::H),
            self.registers.get_r8_value(R8::L),
            self.registers.get_sp(),
            self.pc,
            self.bus.borrow().read_byte(self.pc),
            self.bus.borrow().read_byte(self.pc.wrapping_add(1)),
            self.bus.borrow().read_byte(self.pc.wrapping_add(2)),
            self.bus.borrow().read_byte(self.pc.wrapping_add(3)),
        )
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            registers: Registers::default(),
            bus: Rc::new(RefCell::new(Mmu::default())),
            pc: 0x0100,
            ime: false,
            ime_delay: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::io::Write;
    use std::rc::Rc;

    fn run_rom_test(rom_path: &str, logfile_name: &str) {
        let log_dir = Path::new("logfiles");
        if !log_dir.exists() {
            fs::create_dir_all(log_dir).expect("Failed to create `logfiles` directory");
        }

        let rom_data = fs::read(rom_path).expect("Failed to read ROM file");
        let bus = Rc::new(RefCell::new(Mmu::new(&rom_data)));
        let mut cpu = Cpu::new(bus.clone());
        let mut logfile = fs::File::create(format!("logfiles/{}", logfile_name))
            .expect("Failed to create logfile");

        let mut last_pc = 0xFFFF;
        let mut same_pc_count = 0;

        loop {
            writeln!(logfile, "{}", cpu).expect("Failed to write to logfile");
            cpu.step();

            if cpu.pc == last_pc {
                same_pc_count += 1;
            } else {
                same_pc_count = 0;
            }

            last_pc = cpu.pc;

            if same_pc_count > 100 {
                break; // Assume program has finished
            }
        }
    }

    #[ignore]
    #[test]
    fn test_rom_01_special() {
        run_rom_test("roms/individual/01-special.gb", "logfile-01-special");
    }

    #[ignore]
    #[test]
    fn test_rom_02_interrupts() {
        run_rom_test("roms/individual/02-interrupts.gb", "logfile-02-interrupts");
    }

    #[ignore]
    #[test]
    fn test_rom_03_op_sp_hl() {
        run_rom_test("roms/individual/03-op sp,hl.gb", "logfile-03-op-sp-hl");
    }

    #[ignore]
    #[test]
    fn test_rom_04_op_r_imm() {
        run_rom_test("roms/individual/04-op r,imm.gb", "logfile-04-op-r-imm");
    }

    #[ignore]
    #[test]
    fn test_rom_05_op_rp() {
        run_rom_test("roms/individual/05-op rp.gb", "logfile-05-op-rp");
    }

    #[ignore]
    #[test]
    fn test_rom_06_ld_r_r() {
        run_rom_test("roms/individual/06-ld r,r.gb", "logfile-06-ld-r-r");
    }

    #[ignore]
    #[test]
    fn test_rom_07_jr_jp_call_ret_rst() {
        run_rom_test(
            "roms/individual/07-jr,jp,call,ret,rst.gb",
            "logfile-07-jr-jp-call-ret-rst",
        );
    }

    #[ignore]
    #[test]
    fn test_rom_08_misc_instrs() {
        run_rom_test(
            "roms/individual/08-misc instrs.gb",
            "logfile-08-misc-instrs",
        );
    }

    #[ignore]
    #[test]
    fn test_rom_09_op_r_r() {
        run_rom_test("roms/individual/09-op r,r.gb", "logfile-09-op-r-r");
    }

    #[ignore]
    #[test]
    fn test_rom_10_bit_ops() {
        run_rom_test("roms/individual/10-bit ops.gb", "logfile-10-bit-ops");
    }

    #[ignore]
    #[test]
    fn test_rom_11_op_a_hl() {
        run_rom_test("roms/individual/11-op a,(hl).gb", "logfile-11-op-a-hl");
    }
}
