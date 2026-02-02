#![allow(unused_variables)]
#![allow(dead_code)]

pub mod interrupt;
pub mod mbc;
pub mod timers;
pub mod oam;

use self::timers::Timers;
use crate::mmu::interrupt::Interrupt;
use crate::mmu::interrupt::InterruptController;
use crate::mmu::mbc::Mbc;
use crate::mmu::oam::Oam;

#[derive(PartialEq, Eq, Debug)]
pub enum MemoryRegion {
    Mbc,             // 0x000-0x7FFF: read-only
    Vram,            // 0x8000-0x9FFF
    ERam,            // 0xA000-0xBFFF
    Wram,            // 0xC000-0xDFFF
    Mram,            // 0xE000-0xFDFF: mirror of C000-DDFF
    Oam,             // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable,        // 0xFEA0-0xFEFF
    InterruptFlag,   // 0xFF0F: Interruption Flag: Inside IO
    Timers,          // 0xFF04-0xFF07
    Io,              // 0xFF00-0xFF7F
    HRam,            // 0xFF80-0xFFFE
    InterruptEnable, // 0xFFFF : Interruption Enable
}

impl MemoryRegion {
    pub fn from(addr: u16) -> Self {
        match addr {
            0x0000..=0x7FFF => MemoryRegion::Mbc,
            0x8000..=0x9FFF => MemoryRegion::Vram,
            0xA000..=0xBFFF => MemoryRegion::ERam,
            0xC000..=0xDFFF => MemoryRegion::Wram,
            0xE000..=0xFDFF => MemoryRegion::Mram,
            0xFE00..=0xFE9F => MemoryRegion::Oam,
            0xFEA0..=0xFEFF => MemoryRegion::Unusable,
            0xFF04..=0xFF07 => MemoryRegion::Timers,
            0xFF0F => MemoryRegion::InterruptFlag,
            0xFF00..=0xFF7F => MemoryRegion::Io,
            0xFF80..=0xFFFE => MemoryRegion::HRam,
            0xFFFF => MemoryRegion::InterruptEnable,
        }
    }

    pub fn to_address(&self) -> u16 {
        match self {
            MemoryRegion::Mbc => 0x0000,
            MemoryRegion::Vram => 0x8000,
            MemoryRegion::ERam => 0xA000,
            MemoryRegion::Wram => 0xC000,
            MemoryRegion::Mram => 0xE000,
            MemoryRegion::Oam => 0xFE00,
            MemoryRegion::Unusable => 0xFEA0,
            MemoryRegion::Timers => 0xFF04,
            MemoryRegion::InterruptFlag => 0xFF0F,
            MemoryRegion::Io => 0xFF00,
            MemoryRegion::HRam => 0xFF80,
            MemoryRegion::InterruptEnable => 0xFFFF,
        }
    }
}

pub struct Mmu {
    data: [u8; 0x10000], // 0xFFFF (65535) + 1 = 0x10000 (65536)
    cart: Mbc,
    interrupts: InterruptController,
    timers: Timers,
    oam: Oam,
}

impl Mmu {
    pub fn new(rom_image: &[u8]) -> Self {
        let mmu = Mmu {
            data: [0; 0x10000],
            cart: Mbc::new(rom_image),
            interrupts: InterruptController::new(),
            timers: Timers::default(),
            oam: Oam::default(),
        };

       mmu
    }

    pub fn tick_timers(&mut self) {
        if self.timers.tick() {
            let interrupt_flags_addr = MemoryRegion::InterruptFlag.to_address();
            let mut interrupt_flags = self.read_byte(interrupt_flags_addr);
            interrupt_flags |= 0b100;
            self.write_byte(interrupt_flags_addr, interrupt_flags);
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match MemoryRegion::from(addr) {
            MemoryRegion::Mbc => self.cart.read(addr),
            MemoryRegion::Timers => self.timers.read_byte(addr),
            MemoryRegion::Mram => {
                let mirror = addr - 0x2000;

                self.data[mirror as usize]
            }
            MemoryRegion::Oam => self.oam.read(addr),
            MemoryRegion::Unusable => 0xFF,
            MemoryRegion::InterruptFlag => self.interrupts.read_interrupt_flag(),
            MemoryRegion::InterruptEnable => self.interrupts.read_interrupt_enable(),
            _ => self.data[addr as usize],
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match MemoryRegion::from(addr) {
            MemoryRegion::Mbc => self.cart.write(addr, val),
            MemoryRegion::Mram => {
                let mirror = addr - 0x2000;

                self.data[mirror as usize] = val;
            }
            MemoryRegion::Timers => {
                self.timers.write_byte(addr, val);
            }
            MemoryRegion::Oam => self.oam.write(addr, val),
            MemoryRegion::Unusable => {}
            MemoryRegion::InterruptFlag => self.interrupts.write_interrupt_flag(val),
            MemoryRegion::InterruptEnable => self.interrupts.write_interrupt_enable(val),
            _ => self.data[addr as usize] = val,
        }
    }

    pub fn read_interrupt_enable(&self) -> u8 {
        self.interrupts.read_interrupt_enable()
    }

    pub fn read_interrupt_flag(&self) -> u8 {
        self.interrupts.read_interrupt_flag()
    }

    pub fn interrupts_next_request(&self) -> Option<Interrupt> {
        self.interrupts.next_request()
    }

    pub fn interrupts_clear_request(&mut self, interrupt: Interrupt) {
        self.interrupts.clear_request(interrupt);
    }

    pub fn interrupts_request(&mut self, interrupt: Interrupt) {
        self.interrupts.request(interrupt);
    }

    pub fn get_oam(&self) -> &Oam {
        &self.oam
    }
}

impl Default for Mmu {
    fn default() -> Self {
        Mmu::new(&[])
    }
}

// In mmu.rs
#[cfg(test)]
mod tests {
    use super::{MemoryRegion, Mmu};

    #[test]
    fn mmu_routes_reads_and_writes() {
        let rom = vec![0x12, 0x34, 0x56, 0x78];
        let mut mmu = Mmu::new(&rom);

        // Reading from ROM region gives you the first bank data
        assert_eq!(mmu.read_byte(0x0000), 0x12);
        assert_eq!(mmu.read_byte(0x0001), 0x34);

        // Write to WRAM region and read back
        mmu.write_byte(0xC000, 0xAB);
        assert_eq!(mmu.read_byte(0xC000), 0xAB);
    }

    #[test]
    fn memory_region_from_addr() {
        assert_eq!(MemoryRegion::from(0x0000), MemoryRegion::Mbc);
        assert_eq!(MemoryRegion::from(0x8000), MemoryRegion::Vram);
        assert_eq!(MemoryRegion::from(0xA123), MemoryRegion::ERam);
        assert_eq!(MemoryRegion::from(0xC123), MemoryRegion::Wram);
        assert_eq!(MemoryRegion::from(0xE123), MemoryRegion::Mram);
        assert_eq!(MemoryRegion::from(0xFE50), MemoryRegion::Oam);
        assert_eq!(MemoryRegion::from(0xFEA0), MemoryRegion::Unusable);
        assert_eq!(MemoryRegion::from(0xFF0F), MemoryRegion::InterruptFlag);
        assert_eq!(MemoryRegion::from(0xFF10), MemoryRegion::Io);
        assert_eq!(MemoryRegion::from(0xFF80), MemoryRegion::HRam);
        assert_eq!(MemoryRegion::from(0xFFFF), MemoryRegion::InterruptEnable);
    }

    // MRAM ECHO RAM
    #[test]
    fn echo_ram_mirror() {
        let mut mmu = Mmu::new(&[]);

        // Write to Work RAM (0xC000) and read from Echo RAM (0xE000)
        mmu.write_byte(0xC000, 0xAA);
        assert_eq!(mmu.read_byte(0xE000), 0xAA);

        // Write to Echo RAM and read from Work RAM
        mmu.write_byte(0xE010, 0xBB);
        assert_eq!(mmu.read_byte(0xC010), 0xBB);
    }

    // UNUSABLE REGION
    #[test]
    fn unusable_region_behavior() {
        let mut mmu = Mmu::new(&[]);

        // Unusable region reads back as 0xFF
        let base = 0xFEA0;
        assert_eq!(mmu.read_byte(base), 0xFF);
        assert_eq!(mmu.read_byte(base + 0x1F), 0xFF);

        // Writes to unusable region are ignored (reads still 0xFF)
        mmu.write_byte(base, 0x00);
        mmu.write_byte(base + 0x1F, 0x12);
        assert_eq!(mmu.read_byte(base), 0xFF);
        assert_eq!(mmu.read_byte(base + 0x1F), 0xFF);
    }
}
