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

use std::fmt;
use std::sync::{Arc, RwLock};

use crate::cpu::registers::{R8, R16, Registers};
use crate::mmu::Mmu;

const BLOCK_MASK: u8 = 0b11000000;

#[derive(Debug, Clone, Copy, PartialEq)]
enum StepStatus {
    Continue,
    Halted,
}

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub bus: Arc<RwLock<Mmu>>,
    pub ime: bool,
    pub ime_delay: bool, // mimic hardware delay in EI
    pub halted: bool,    // for HALT instruction
    pub halt_bug: bool,
    tick_to_wait: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            registers: Registers::default(),
            bus: Arc::new(RwLock::new(Mmu::default())),
            pc: 0x0100,
            ime: false,
            ime_delay: false,
            halted: false,
            halt_bug: false,
            tick_to_wait: 0,
        }
    }
}

impl Cpu {
    pub fn new(bus: Arc<RwLock<Mmu>>) -> Self {
        Cpu {
            pc: 0x0100,
            bus,
            ..Default::default()
        }
    }

    fn execute_instruction(&mut self, instruction: u8) -> u8 {
        let block = (instruction & BLOCK_MASK) >> 6;
        match block {
            0b00 => block0::execute_instruction_block0(self, instruction),
            0b01 => block1::execute_instruction_block1(self, instruction),
            0b10 => block2::execute_instruction_block2(self, instruction),
            0b11 => block3::execute_instruction_block3(self, instruction),
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self) {
        if self.tick_to_wait > 0 {
            self.tick_to_wait -= 1;
        } else {
            self.tick_to_wait = self.step();
        }
    }

    fn handle_halt_state(&mut self) -> StepStatus {
        if self.halted {
            let bus = self.bus.read().unwrap();
            let iflag = bus.read_interrupt_flag();
            let ienable = bus.read_interrupt_enable();

            if ienable & iflag == 0 {
                return StepStatus::Halted;
            }

            self.halted = false;

            if !self.ime {
                self.halt_bug = true;
            }
        }
        StepStatus::Continue
    }

    fn handle_ime_state(&mut self) -> StepStatus {
        if self.ime {
            let mut bus = self.bus.write().unwrap();
            if let Some(interrupt) = bus.interrupts_next_request() {
                self.ime = false;
                bus.interrupts_clear_request(interrupt);

                let ret_addr = self.pc;

                let sp1 = self.registers.get_sp().wrapping_sub(1);
                self.registers.set_sp(sp1);
                bus.write_byte(sp1, (ret_addr >> 8) as u8);

                let sp2 = sp1.wrapping_sub(1);
                self.registers.set_sp(sp2);
                bus.write_byte(sp2, (ret_addr & 0xFF) as u8);

                self.pc = interrupt.vector();
                StepStatus::Halted
            } else {
                StepStatus::Continue
            }
        } else {
            StepStatus::Continue
        }
    }

    fn handle_halt_bug(&mut self) {
        if self.halt_bug {
            self.pc = self.pc.wrapping_sub(1);
            self.halt_bug = false;
        }
    }

    fn handle_ime_delay(&mut self) {
        if self.ime_delay {
            self.ime = true;
            self.ime_delay = false;
        }
    }

    pub fn step(&mut self) -> u8 {
        if self.handle_halt_state() == StepStatus::Halted {
            return 4;
        }
        if self.handle_ime_state() == StepStatus::Halted {
            return 5;
        }

        let instruction_byte = self.bus.read().unwrap().read_byte(self.pc);
        let tick_to_wait = self.execute_instruction(instruction_byte);

        self.handle_halt_bug();
        self.handle_ime_delay();

        tick_to_wait
    }

    pub fn get_r8_value(&self, register: R8) -> u8 {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.read().unwrap().read_byte(addr)
            }
            _ => self.registers.get_r8_value(register),
        }
    }

    pub fn set_r8_value(&mut self, register: R8, value: u8) {
        match register {
            R8::HLIndirect => {
                let addr = self.registers.get_r16_value(R16::HL);
                self.bus.write().unwrap().write_byte(addr, value);
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
            self.bus.read().unwrap().read_byte(self.pc),
            self.bus.read().unwrap().read_byte(self.pc.wrapping_add(1)),
            self.bus.read().unwrap().read_byte(self.pc.wrapping_add(2)),
            self.bus.read().unwrap().read_byte(self.pc.wrapping_add(3)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    use crate::mmu::interrupt::Interrupt;

    // interrupts tests
    #[test]
    fn test_cpu_services_timer_interrupt() {
        // 1) Set up MMU and manually enable/request the Timer interrupt
        let mut mmu = Mmu::new(&[]);
        // Enable only Timer (bit 2) in IE
        mmu.write_byte(0xFFFF, Interrupt::Timer as u8);
        // Request Timer by writing to IF
        mmu.write_byte(0xFF0F, Interrupt::Timer as u8);

        // 2) Create CPU with that MMU
        let bus = Arc::new(RwLock::new(mmu));
        let mut cpu = Cpu::new(bus.clone());

        // 3) Initialize PC and SP
        cpu.pc = 0x1234;
        cpu.registers.set_sp(0xFFFE);
        // Allow interrupts immediately
        cpu.ime = true;

        // 4) Perform one step: should service the Timer interrupt
        cpu.step();

        // 5) After service, SP must have decreased by 2
        assert_eq!(cpu.registers.get_sp(), 0xFFFC);

        // 6) Check the two bytes on the stack (little-endian: low then high)
        let mmu = bus.read().unwrap();
        // Low byte of 0x1234 at address 0xFFFC
        assert_eq!(mmu.read_byte(0xFFFC), 0x34);
        // High byte of 0x1234 at address 0xFFFD
        assert_eq!(mmu.read_byte(0xFFFD), 0x12);

        // 7) CPU should have jumped to the Timer vector (0x50)
        assert_eq!(cpu.pc, Interrupt::Timer.vector());

        // 8) IME should now be cleared
        assert!(!cpu.ime);

        // 9) IF’s Timer bit must have been cleared
        assert_eq!(mmu.read_byte(0xFF0F) & (1 << (Interrupt::Timer as u8)), 0);
    }

    // HALT tests
    #[test]
    fn test_halt_opcode_sets_halted_and_advances_pc() {
        // Setup: place a HALT (0x76) at address 0x200
        let mut mmu = Mmu::new(&[]);
        mmu.write_byte(0x8000, 0x76);
        let bus = Arc::new(RwLock::new(mmu));

        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x8000;

        // Execute one step → should see the HALT instruction
        cpu.step();

        // After HALT: halted flag set, PC advanced by 1
        assert!(cpu.halted, "CPU should be halted after executing HALT");
        assert_eq!(cpu.pc, 0x8001, "PC must point past the HALT opcode");
    }

    #[test]
    fn test_step_halt_stays_halted_without_interrupt() {
        // If halted==true and no pending interrupt, step() must do nothing
        let mmu = Mmu::new(&[]);
        let bus = Arc::new(RwLock::new(mmu));
        let mut cpu = Cpu::new(bus);

        cpu.halted = true;
        cpu.pc = 0x123;
        cpu.ime = false; // IME doesn't matter when no interrupt
        // Ensure IF & IE = 0
        cpu.step();
        assert!(cpu.halted, "Still halted if no interrupt pending");
        assert_eq!(cpu.pc, 0x123, "PC must not change when halted and idle");
    }

    #[test]
    fn test_step_halt_wakes_without_servicing_when_ime_false() {
        // If halted==true and an interrupt is pending but IME==false,
        // CPU should wake (halted→false) but *not* service the interrupt.
        let mut mmu = Mmu::new(&[]);
        // Make a pending interrupt: Timer bit in IF and IE
        mmu.write_byte(0xFF0F, Interrupt::Timer as u8);
        mmu.write_byte(0xFFFF, Interrupt::Timer as u8);
        // Also put a dummy opcode (0x00 = NOP) at PC so we can see it execute.
        mmu.write_byte(0x300, 0x00);
        let bus = Arc::new(RwLock::new(mmu));
        let mut cpu = Cpu::new(bus.clone());
        cpu.pc = 0x300;
        cpu.registers.set_sp(0xFFFE);
        cpu.halted = true;
        cpu.ime = false; // Master-enable off

        cpu.step();

        // Halt should clear, but with IME=0 and a pending interrupt the halt-bug fires:
        // we expect the very next byte (0x00 at 0x300) to repeat, so PC stays at 0x300.
        assert!(!cpu.halted, "CPU must wake up when interrupt pending");
        assert_eq!(
            cpu.pc, 0x300,
            "PC should *not* advance thanks to the halt bug"
        );
        // And IF should remain unchanged, since IME==false means no service
        let mmu = bus.read().unwrap();
        assert_ne!(
            mmu.read_byte(0xFF0F) & (Interrupt::Timer as u8),
            0,
            "IF should still contain the pending bit when IME is false"
        );
    }

    #[test]
    fn test_step_halt_wake_and_service_when_ime_true() {
        // Combination of HALT wake-up + interrupt dispatch in one step:
        let mut mmu = Mmu::new(&[]);
        mmu.write_byte(0xFF0F, Interrupt::Timer as u8);
        mmu.write_byte(0xFFFF, Interrupt::Timer as u8);
        let bus = Arc::new(RwLock::new(mmu));
        let mut cpu = Cpu::new(bus.clone());

        cpu.pc = 0x400;
        cpu.registers.set_sp(0xFFFE);
        cpu.halted = true;
        cpu.ime = true;

        cpu.step();

        // Should have pushed return addr 0x400, jumped to 0x50, cleared halted & IME
        assert_eq!(cpu.registers.get_sp(), 0xFFFC);
        let mmu = bus.read().unwrap();
        assert_eq!(mmu.read_byte(0xFFFC), 0x00, "low byte of 0x0400");
        assert_eq!(mmu.read_byte(0xFFFD), 0x04, "high byte of 0x0400");
        assert_eq!(cpu.pc, Interrupt::Timer.vector());
        assert!(!cpu.ime, "IME must be cleared after servicing");
        assert_eq!(
            mmu.read_byte(0xFF0F) & (Interrupt::Timer as u8),
            0,
            "IF Timer bit must be cleared"
        );
    }

    #[test]
    fn test_halt_bug_repeats_next_byte() {
        use crate::cpu::registers::R8;
        use crate::mmu::interrupt::Interrupt;

        // 1) Lay out a tiny program in WRAM (0xC000..):
        //      0xC000: 0x76       ; HALT
        //      0xC001: 0x04       ; INC B
        let mut mmu = Mmu::new(&[]);
        mmu.write_byte(0xC000, 0x76);
        mmu.write_byte(0xC001, 0x04);

        // 2) Create CPU, point it at our “program”
        let bus = Arc::new(RwLock::new(mmu));
        let mut cpu = Cpu::new(bus.clone());
        cpu.pc = 0xC000;
        cpu.registers.set_r8_value(R8::B, 0);

        // 3) Trigger the halt bug: IME=0, and set IF & IE so (IE&IF)!=0
        {
            let mut mmu = bus.write().unwrap();
            mmu.write_byte(0xFFFF, Interrupt::Timer as u8); // IE
            mmu.write_byte(0xFF0F, Interrupt::Timer as u8); // IF
        }
        cpu.ime = false;

        // 4) Step 1: execute the HALT itself (sets `halted`, moves PC→0xC001)
        cpu.step();
        assert!(cpu.halted, "after HALT, CPU should be halted");
        assert_eq!(cpu.pc, 0xC001, "PC must advance past HALT");

        // 5) Step 2: wake+bug → should execute the INC B at 0xC001
        cpu.step();
        // B should have gone from 0 → 1:
        assert_eq!(cpu.registers.get_r8_value(R8::B), 1);

        // 6) Step 3: with no more HALT state, just execute INC B again
        cpu.step();
        // B should now be 2, confirming the “repeat” of that byte:
        assert_eq!(cpu.registers.get_r8_value(R8::B), 2);
    }

    // roms tests
    fn run_rom_test(rom_path: &str, logfile_name: &str) {
        let log_dir = Path::new("logfiles");
        if !log_dir.exists() {
            fs::create_dir_all(log_dir).expect("Failed to create `logfiles` directory");
        }

        let rom_data = fs::read(rom_path).expect("Failed to read ROM file");
        let bus = Arc::new(RwLock::new(Mmu::new(&rom_data)));
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
