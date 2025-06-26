#![allow(unused_variables)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::Cpu;
use crate::memory::MemoryBus;
use crate::ppu::Ppu;

#[derive(Default)]
pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Rc<RefCell<MemoryBus>>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        let bus = Rc::new(RefCell::new(MemoryBus::new(rom)));
        let cpu = Cpu::new(bus.clone());
        let ppu = Ppu::new(bus.clone());

        GameBoy { cpu, bus, ppu }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
            // Here you could add a delay or frame rendering logic
        }
    }
}
