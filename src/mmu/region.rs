#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(PartialEq, Eq)]
pub enum MemoryRegion {
    Rbank,       // 0x000-0x7FFF: read-only
    Vram,        // 0x8000-0x9FFF
    ExternalRam, // 0xA000-0xBFFF
    Wram,        // 0xC000-0xDFFF
    Echo,        // 0xE000-0xFDFF: mirror of C000-DDFF
    Oam,         // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable,    // 0xFEA0-0xFEFF
    Io,          // 0xFF00-0xFF7F
    HRam,        // 0xFF80-0xFFFE
    Ie,          // 0xFFFF-0xFFFF
}

// TODO refacto in order to have multiple structs instead of one
impl MemoryRegion {
    pub fn from(addr: u16) -> Self {
        match addr {
            0x0000..=0x7FFF => MemoryRegion::Rbank,
            0x8000..=0x9FFF => MemoryRegion::Vram,
            0xA000..=0xBFFF => MemoryRegion::ExternalRam,
            0xC000..=0xDFFF => MemoryRegion::Wram,
            0xE000..=0xFDFF => MemoryRegion::Echo,
            0xFE00..=0xFE9F => MemoryRegion::Oam,
            0xFEA0..=0xFEFF => MemoryRegion::Unusable,
            0xFF00..=0xFF7F => MemoryRegion::Io,
            0xFF80..=0xFFFE => MemoryRegion::HRam,
            0xFFFF => MemoryRegion::Ie,
        }
    }

    pub fn is_readable(&self) -> bool {
        !matches!(self, MemoryRegion::Unusable)
    }
    pub fn is_writable(&self) -> bool {
        !matches!(self, MemoryRegion::Unusable)
    }

    pub fn translate_into_physical_address(&self, addr: u16) -> u16 {
        match self {
            MemoryRegion::Echo => addr - 0x2000,
            _ => addr,
        }
    }
}
