#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::{Arc, RwLock};

use std::sync::Mutex;

use crate::cpu::Cpu;
use crate::cpu::registers::{R8, Registers};
use crate::gui::KeyInput;
use crate::mmu::interrupt::Interrupt;
use crate::mmu::mbc::Mbc;
use crate::mmu::Mmu;
use crate::ppu::Ppu;

const FRAME_CYCLES: u32 = 70224;
const WIN_SIZE_X: usize = 160; // Window size in X direction
const WIN_SIZE_Y: usize = 144; // Window size in Y direction
const VBLANK_SIZE: usize = 10; // VBlank size in lines

pub struct GameBoy<T: Mbc> {
    pub cpu: Cpu<T>,
    pub ppu: Ppu<T>,
    pub bus: Arc<RwLock<Mmu<T>>>,
    pub image: Arc<Mutex<Vec<u8>>>,
}

impl<T: Mbc>  GameBoy<T> {
    pub fn new(rom: Vec<u8>, boot_rom: Option<[u8; 0x0100]>, image: Arc<Mutex<Vec<u8>>>) -> Result<GameBoy<T>, String> {
        let bus_ref = Arc::new(RwLock::new(Mmu::<T>::new(&rom)?));

        if let Some(boot_rom) = boot_rom {
            let mut mmu = bus_ref.write().unwrap();
            mmu.load_boot_rom(boot_rom);
        }

        let cpu = Cpu::<T>::new(bus_ref.clone());
        let ppu = Ppu::<T>::new(bus_ref.clone());

        Ok(GameBoy { cpu, bus: bus_ref, ppu, image })
    }

    pub fn simulate_boot_rom_effect(&mut self) {
        self.cpu.set_r8_value(R8::A, 0x01);
        self.cpu.set_r8_value(R8::B, 0xFF);
        self.cpu.set_r8_value(R8::C, 0x13);
        self.cpu.set_r8_value(R8::D, 0x00);
        self.cpu.set_r8_value(R8::E, 0xC1);
        self.cpu.set_r8_value(R8::H, 0x84);
        self.cpu.set_r8_value(R8::L, 0x03);
        self.cpu.pc = 0x0100;
        self.cpu.registers.set_sp(0xFFFE);

        let mut bus = self.bus.write().unwrap();

        bus.write_byte(0xFF00, 0xCF);
        bus.write_byte(0xFF01, 0x00);
        bus.write_byte(0xFF02, 0x7E);
        bus.write_byte(0xFF04, 0x18);
        bus.write_byte(0xFF05, 0x00);
        bus.write_byte(0xFF06, 0x00);
        bus.write_byte(0xFF07, 0xF8);
        bus.write_byte(0xFF0F, 0xE1);
        bus.write_byte(0xFF10, 0x80);
        bus.write_byte(0xFF11, 0xBF);
        bus.write_byte(0xFF12, 0xF3);
        bus.write_byte(0xFF13, 0xFF);
        bus.write_byte(0xFF14, 0xBF);
        bus.write_byte(0xFF16, 0x3F);
        bus.write_byte(0xFF17, 0x00);
        bus.write_byte(0xFF18, 0xFF);
        bus.write_byte(0xFF19, 0xBF);
        bus.write_byte(0xFF1A, 0x7F);
        bus.write_byte(0xFF1B, 0xFF);
        bus.write_byte(0xFF1C, 0x9F);
        bus.write_byte(0xFF1D, 0xFF);
        bus.write_byte(0xFF1E, 0xBF);
        bus.write_byte(0xFF20, 0xFF);
        bus.write_byte(0xFF21, 0x00);
        bus.write_byte(0xFF22, 0x00);
        bus.write_byte(0xFF23, 0xBF);
        bus.write_byte(0xFF24, 0x77);
        bus.write_byte(0xFF25, 0xF3);
        bus.write_byte(0xFF26, 0xF1);
        bus.write_byte(0xFF40, 0x91);
        bus.write_byte(0xFF41, 0x81);
        bus.write_byte(0xFF42, 0x00);
        bus.write_byte(0xFF43, 0x00);
        bus.write_byte(0xFF44, 0x91);
        bus.write_byte(0xFF45, 0x00);
        bus.write_byte(0xFF46, 0xFF);
        bus.write_byte(0xFF47, 0xFC);
        bus.write_byte(0xFF4A, 0x00);
        bus.write_byte(0xFF4B, 0x00);
        bus.write_byte(0xFFFF, 0x00);
    }

    pub fn manage_input(&mut self, key_input: &KeyInput) {
        if key_input.into() {
            let mut bus = self.bus.write().unwrap();
            bus.interrupts_request(Interrupt::Joypad);
        }
        let bus = self.bus.read().unwrap();
        let mut p1_joypad = bus.read_byte(0xFF00);
        let (slcted_buttons, slcted_pad) = {
            (p1_joypad & 0b0010_0000 == 0, p1_joypad & 0b0001_0000 == 0)
        };

        p1_joypad &= 0b0011_0000;
        if slcted_pad {
            if key_input.down_pushed    { p1_joypad &= 0b1111_0111; }
            if key_input.up_pushed      { p1_joypad &= 0b1111_1011; }
            if key_input.left_pushed    { p1_joypad &= 0b1111_1101; }
            if key_input.right_pushed   { p1_joypad &= 0b1111_1110; }
        }

        if slcted_buttons {
            if key_input.start_pushed   { p1_joypad &= 0b1111_0111; }
            if key_input.select_pushed  { p1_joypad &= 0b1111_1011; }
            if key_input.b_pushed       { p1_joypad &= 0b1111_1101; }
            if key_input.a_pushed       { p1_joypad &= 0b1111_1110; }
        }
        let mut bus = self.bus.write().unwrap();
        bus.write_byte(0xFF00, p1_joypad);
    }

    pub fn run_frame(&mut self, key_input: &KeyInput) -> bool {
        let mut cycles_elapsed = 0;


        while cycles_elapsed < FRAME_CYCLES {
            self.manage_input(key_input);

            // 1. Tick Timers
            self.bus.write().unwrap().tick_timers();

            // 2. Tick CPU
            self.cpu.tick();

            // 3. Tick PPU
            let vblank = self.ppu.tick(1, &mut self.image);

            if vblank {
                return true;
            }

            cycles_elapsed += 1;
        }
        false
    }
}
