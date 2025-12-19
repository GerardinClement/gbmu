#![allow(unused_variables)]
#![allow(dead_code)]

use std::sync::{Arc, RwLock};

use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::Ppu;

const FRAME_CYCLES: u32 = 70224;
const NB_OF_DOT_PER_TICK: u32 = 4;
const WIN_SIZE_X: usize = 160; // Window size in X direction
const WIN_SIZE_Y: usize = 144; // Window size in Y direction
const VBLANK_SIZE: usize = 10; // VBlank size in lines

#[derive(Default)]
pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Arc<RwLock<Mmu>>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        let bus = Arc::new(RwLock::new(Mmu::new(&rom)));
        let cpu = Cpu::new(bus.clone());
        let ppu = Ppu::new(bus.clone());

        GameBoy { cpu, bus, ppu }
    }

    pub fn run_frame(&mut self) -> Vec<u8> {
        let mut cycles_this_frame = 0;
        let mut frame = vec![0; WIN_SIZE_X * WIN_SIZE_Y * 3];

        while cycles_this_frame < FRAME_CYCLES {
            self.bus.write().unwrap().tick_timers();
            self.cpu.tick();
            cycles_this_frame += 1;

            self.ppu.update_registers();
            self.ppu.render_frame(&mut frame)
        }
        frame
    }
}
