#![allow(unused_variables)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::Ppu;

#[derive(Default)]
pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Rc<RefCell<Mmu>>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        let bus = Rc::new(RefCell::new(Mmu::new(&rom)));
        let cpu = Cpu::new(bus.clone());
        let ppu = Ppu::new(bus.clone());

        GameBoy { cpu, bus, ppu }
    }

    pub fn run_frame(&mut self) -> Vec<u8> {
        let mut cycles_this_frame = 0;

        while cycles_this_frame < 70224 {
            self.cpu.step();
            cycles_this_frame += 1;
            self.ppu.update_registers();
        }

        self.ppu.render_frame()
    }
}
