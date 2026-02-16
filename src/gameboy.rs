#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::{Arc, RwLock};

use std::sync::Mutex;

use crate::cpu::Cpu;
use crate::gui::KeyInput;
use crate::mmu::Mmu;
use crate::ppu::Ppu;

const FRAME_CYCLES: u32 = 70224;
const WIN_SIZE_X: usize = 160; // Window size in X direction
const WIN_SIZE_Y: usize = 144; // Window size in Y direction
const VBLANK_SIZE: usize = 10; // VBlank size in lines

#[derive(Default)]
pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Arc<RwLock<Mmu>>,
    pub image: Arc<Mutex<Vec<u8>>>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>, boot_rom: [u8; 0x0100], image: Arc<Mutex<Vec<u8>>>) -> Self {
        let bus = Arc::new(RwLock::new(Mmu::new(&rom)));

        {
            let mut mmu = bus.write().unwrap();
            mmu.load_boot_rom(boot_rom);
        }

        let cpu = Cpu::new(bus.clone());
        let ppu = Ppu::new(bus.clone());

        GameBoy { cpu, bus, ppu, image }
    }

    pub fn run_frame(&mut self, key_input: &KeyInput) -> bool {
        let mut cycles_elapsed = 0;

        while cycles_elapsed < FRAME_CYCLES {
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
