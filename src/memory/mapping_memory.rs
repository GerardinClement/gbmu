
/*
Step 5: Cartridge & MBC
├── Change Memory::new to take rom_image: Vec<u8>
│   └── Split into rom_banks: Vec<[u8; 0x4000]>
├── Add MBC state (current_rom_bank, ram_enabled, current_ram_bank)
├── In read_byte:
│   ├─ if RomBank0 → return rom_banks[0][addr]
│   ├─ if RomBank1N → return rom_banks[current_rom_bank][addr-0x4000]
│   ├─ if ExternalRam → return ext_ram[current_ram_bank][addr-0xA000] (if ram_enabled)
│   └─ else → existing data/regions
├── In write_byte:
│   ├─ if RomBank0 or RomBank1N → self.mbc.handle_write(addr, val)
│   ├─ if ExternalRam → ext_ram[...] = val (if ram_enabled)
│   └─ else → existing data/regions
└── Tests:
    ├─ Bank 1 read/write
    ├─ Bank switch via writes to 0x2000–0x3FFF
    ├─ RAM enable at 0x0000–0x1FFF
    └─ External RAM bank switch 0x4000–0x5FFF
*/

#[derive(PartialEq, Eq)]
pub(crate) enum MemoryRegion {
    RomBank0, // 0x000-0x3FFF: read-only
    RomBank1N, // 0x4000-0x7FFF
    Vram, // 0x8000-0x9FFF
    ExternalRam, // 0xA000-0xBFFF
    WorkRamBank0, // 0xC000-0xCFFF
    WorkRamBank1_7, // 0xD000-0xDFFF
    Echo, // 0xE000-0xFDFF: mirror of C000-DDFF
    OAM, // 0xFE00-0xFE9F: Sprite Attribute Table
    Unusable, // 0xFEA0-0xFEFF
    IO, // 0xFF00-0xFF7F
    HRam, // 0xFF80-0xFFFE
    IE, // 0xFFFF-0xFFFF
}

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
            0xFE00..=0xFE9F => MemoryRegion::OAM,
            0xFEA0..=0xFEFF => MemoryRegion::Unusable,
            0xFF00..=0xFF7F => MemoryRegion::IO,
            0xFF80..=0xFFFE => MemoryRegion::HRam,
            0xFFFF => MemoryRegion::IE,
        }
    }

    pub(crate) fn is_readable(&self) -> bool { !matches!(self, MemoryRegion::Unusable) }
    pub(crate) fn is_writable(&self) -> bool { !matches!(self, MemoryRegion::RomBank0 | MemoryRegion::Unusable) }

    pub(crate) fn translate_into_physical_address(&self, addr: u16) -> u16 {
        match self {
            MemoryRegion::Echo => addr - 0x2000,
            _ => addr,
        }
    }
}
