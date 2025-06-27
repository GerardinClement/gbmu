#![allow(unused_variables)]
#![allow(dead_code)]

pub mod mbc;

use crate::mmu::mbc::Mbc;

#[derive(PartialEq, Eq)]
pub enum MemoryRegion {
    Mbc,        // 0x000-0x7FFF: read-only
    Vram,           // 0x8000-0x9FFF
    ERam,           // 0xA000-0xBFFF
    Wram,           // 0xC000-0xDFFF
    Mram,           // 0xE000-0xFDFF: mirror of C000-DDFF
    Oam,            // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable,       // 0xFEA0-0xFEFF
    If,             // 0xFF0F: Interruption Flag: Inside IO
    Io,             // 0xFF00-0xFF7F
    HRam,           // 0xFF80-0xFFFE
    Ie,             // 0xFFFF : Interruption Enable
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
    rom: Mbc,
}

impl Mmu {
    pub fn new(rom_image: Vec<u8>) -> Self {
        let mut data = [0; 0x10000];

        let rom = Mbc::new(&rom_image);

        Mmu {
            data,
            rom,
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.rom.clear();

        let mut bank0 = vec![0; 0x4000];
        for (i, byte) in rom.iter().enumerate() {
            if i >= 0x4000 {
                break;
            }
            bank0[i] = *byte;
        }
        self.rom.push(bank0);

        let mut offset = 0x4000;
        while offset < rom.len() && self.rom.len() < 128 {
            let mut bank = vec![0; 0x4000];
            for (i, byte) in rom[offset..].iter().enumerate() {
                if i >= 0x4000 {
                    break;
                }
                bank[i] = *byte;
            }
            self.rom.push(bank);
            offset += 0x4000;
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr == 0xFF44 {
            return 0x90;
        }

        let region = MemoryRegion::from(addr);

        match region {
            MemoryRegion::Mbc => self.rom.read(addr),
            _ => self.data[addr as usize]
        }

    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        let region = MemoryRegion::from(addr);

        match region {
            MemoryRegion::Mbc => self.rom.write(addr, val),
            _ => self.data[addr as usize] = val
        }
    }
}

impl Default for Mmu {
    fn default() -> Self {
        Mmu::new(&[])
    }
}