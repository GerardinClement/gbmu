#![allow(unused_variables)]
#![allow(dead_code)]

pub mod mbc;
pub mod interrupt;

use crate::mmu::mbc::Mbc;
use crate::mmu::interrupt::InterruptController;

#[derive(PartialEq, Eq, Debug)]
pub enum MemoryRegion {
    Mbc,      // 0x000-0x7FFF: read-only
    Vram,     // 0x8000-0x9FFF
    ERam,     // 0xA000-0xBFFF
    Wram,     // 0xC000-0xDFFF
    Mram,     // 0xE000-0xFDFF: mirror of C000-DDFF
    Oam,      // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable, // 0xFEA0-0xFEFF
    If,       // 0xFF0F: Interruption Flag: Inside IO
    Io,       // 0xFF00-0xFF7F
    HRam,     // 0xFF80-0xFFFE
    Ie,       // 0xFFFF : Interruption Enable
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
            0xFF0F => MemoryRegion::If,
            0xFF00..=0xFF7F => MemoryRegion::Io,
            0xFF80..=0xFFFE => MemoryRegion::HRam,
            0xFFFF => MemoryRegion::Ie,
        }
    }
}

#[derive(Clone)]
pub struct Mmu {
    data: [u8; 0x10000], // 0xFFFF (65535) + 1 = 0x10000 (65536)
    cart: Mbc,
    interrupts: InterruptController,
}

impl Mmu {
    pub fn new(rom_image: &[u8]) -> Self {
        Mmu {
            data: [0; 0x10000],
            cart: Mbc::new(rom_image),
            interrupts: InterruptController::new(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr == 0xFF44 {
            return 0x90;
        }

        match MemoryRegion::from(addr) {
            MemoryRegion::Mbc => self.cart.read(addr),
            MemoryRegion::Mram => {
                let mirror = addr - 0x2000;

                self.data[mirror as usize]
            }
            MemoryRegion::Unusable => 0xFF,
            MemoryRegion::If => self.interrupts.read_if(),
            MemoryRegion::Ie => self.interrupts.read_ie(),
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
            MemoryRegion::Unusable => {},
            MemoryRegion::If => self.interrupts.write_if(val),
            MemoryRegion::Ie => self.interrupts.write_ie(val),
            _ => self.data[addr as usize] = val,
        }
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

        // FF44 is hardcoded
        assert_eq!(mmu.read_byte(0xFF44), 0x90);
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
        assert_eq!(MemoryRegion::from(0xFF0F), MemoryRegion::If);
        assert_eq!(MemoryRegion::from(0xFF10), MemoryRegion::Io);
        assert_eq!(MemoryRegion::from(0xFF80), MemoryRegion::HRam);
        assert_eq!(MemoryRegion::from(0xFFFF), MemoryRegion::Ie);
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
