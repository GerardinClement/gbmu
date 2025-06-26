#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(PartialEq, Eq)]
pub(crate) enum MemoryRegion {
    RomBank0,       // 0x000-0x3FFF: read-only
    RomBank1N,      // 0x4000-0x7FFF
    Vram,           // 0x8000-0x9FFF
    ExternalRam,    // 0xA000-0xBFFF
    WorkRamBank0,   // 0xC000-0xCFFF
    WorkRamBank1_7, // 0xD000-0xDFFF
    Echo,           // 0xE000-0xFDFF: mirror of C000-DDFF
    Oam,            // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable,       // 0xFEA0-0xFEFF
    Io,             // 0xFF00-0xFF7F
    HRam,           // 0xFF80-0xFFFE
    Ie,             // 0xFFFF-0xFFFF
}

// TODO refacto in order to have multiple structs instead of one
impl MemoryRegion {
    // Given any address, pick its region.
    pub(crate) fn from(addr: u16) -> Self {
        match addr {
            0x0000..=0x3FFF => MemoryRegion::RomBank0,
            0x4000..=0x7FFF => MemoryRegion::RomBank1N,
            0x8000..=0x9FFF => MemoryRegion::Vram,
            0xA000..=0xBFFF => MemoryRegion::ExternalRam,
            0xC000..=0xCFFF => MemoryRegion::WorkRamBank0,
            0xD000..=0xDFFF => MemoryRegion::WorkRamBank1_7,
            0xE000..=0xFDFF => MemoryRegion::Echo,
            0xFE00..=0xFE9F => MemoryRegion::Oam,
            0xFEA0..=0xFEFF => MemoryRegion::Unusable,
            0xFF00..=0xFF7F => MemoryRegion::Io,
            0xFF80..=0xFFFE => MemoryRegion::HRam,
            0xFFFF => MemoryRegion::Ie,
        }
    }

    pub(crate) fn is_readable(&self) -> bool {
        !matches!(self, MemoryRegion::Unusable)
    }
    pub(crate) fn is_writable(&self) -> bool {
        !matches!(self, MemoryRegion::Unusable)
    }

    pub(crate) fn translate_into_physical_address(&self, addr: u16) -> u16 {
        match self {
            MemoryRegion::Echo => addr - 0x2000,
            _ => addr,
        }
    }
}
