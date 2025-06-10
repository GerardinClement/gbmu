#![allow(unused_variables)]
#![allow(dead_code)]

use crate::memory::region::MemoryRegion;

pub struct MemoryBus {
    // Flat RAM/IO/etc.
    data: [u8; 0x10000], // 0xFFFF (65535) + 1 = 0x10000 (65536)
    // All 16 KiB ROM banks
    rom_banks: Vec<Vec<u8>>,
    // Which bank is currently mapped into 0x4000-0x7FFF
    current_rom_bank: usize,
}

impl MemoryBus {
    // Init with a full .gb ROM image
    pub fn new(rom: Vec<u8>) -> Self {
        let mut banks = rom
            // slice every 0x4000
            .chunks(0x4000)
            // chunks returns borrowed array so map convert into vectors
            .map(|chunk| chunk.to_vec())
            // gathers into Vec<Vec<u8>>
            .collect::<Vec<_>>();

        if banks.len() == 1 {
            banks.push(vec![0; 0x4000]);
        }

        let mut data =  [0; 0x10000];
        let mut rom_address = 0x4000;

        for byte in rom {
            data[rom_address] = byte;
            rom_address = rom_address.wrapping_add(1);
        }
        
        MemoryBus {
            data: data,
            rom_banks: banks,
            current_rom_bank: 1, // bank 1 is the default switchable region
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let region = MemoryRegion::from(addr);

        if !region.is_readable() {
            return 0xFF;
        }

        if addr == 0xFF44 {
            return 0x90;
        }

        match region {
            MemoryRegion::RomBank0 => {
                return self.rom_banks[0][addr as usize];
            }
            MemoryRegion::RomBank1N => {
                let offset = (addr - 0x4000) as usize;

                return self.rom_banks[self.current_rom_bank][offset];
            }
            _ => {}
        }

        let phys = region.translate_into_physical_address(addr);

        self.data[phys as usize]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        let region = MemoryRegion::from(addr);

        if !region.is_writable() {
            return;
        }

        let phys = region.translate_into_physical_address(addr);

        self.data[phys as usize] = val;
    }
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus { 
            data: [0; 0x10000], 
            rom_banks: Vec::new(), 
            current_rom_bank: 1 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryBus;

    fn make_empty_16kb_rom() -> Vec<u8> {
        vec![0; 0x4000]
    }

    #[test]
    fn rom_bank0_and_bank1_reading() {
        // Build a fake ROM where:
        // - bank 0 is all 0xAA
        // - bank 1 is all 0xBB
        // (each bank is 0x4000 bytes long)
        let mut fake_rom = vec![0xAA; 0x4000];

        fake_rom.extend(vec![0xBB; 0x4000]);

        // Initialize MemoryBus with that ROM
        let bus = MemoryBus::new(fake_rom);
        let bank0: u8 = 0xAA;
        let bank1: u8 = 0xBB;

        assert_eq!(bus.read_byte(0x0000), bank0);
        assert_eq!(bus.read_byte(0x1234), bank0);
        assert_eq!(bus.read_byte(0x3FFF), bank0);

        assert_eq!(bus.read_byte(0x4000), bank1);
        assert_eq!(bus.read_byte(0x4ABC), bank1);
        assert_eq!(bus.read_byte(0x7FFF), bank1);
    }

    #[test]
    fn basic_ram_read_write() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        mem.write_byte(0xC000, 0x42);
        assert_eq!(mem.read_byte(0xC000), 0x42);
    }

    #[test]
    fn vram_read_write() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        mem.write_byte(0x8100, 0x42);
        assert_eq!(mem.read_byte(0x8100), 0x42);
    }

    #[test]
    fn io_registers_read_write() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        let io1 = 0xFF00;
        let io2 = 0xFF7F;
        mem.write_byte(io1, 0x11);
        mem.write_byte(io2, 0x22);
        assert_eq!(mem.read_byte(io1), 0x11);
        assert_eq!(mem.read_byte(io2), 0x22);
    }

    #[test]
    fn rom_is_read_only() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        let rom_addr = 0x2000;
        mem.write_byte(rom_addr, 0x99);
        assert_eq!(mem.read_byte(rom_addr), 0x00);
    }

    #[test]
    fn echo_ram_mirrors_wram() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        let echo = 0xE456;
        let real = 0xC456;
        mem.write_byte(echo, 0xCD);
        assert_eq!(mem.read_byte(real), 0xCD);
    }

    #[test]
    fn echo_ram_mirrors_backwards() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        let echo = 0xE567;
        let real = 0xC567;
        mem.write_byte(real, 0xCF);
        assert_eq!(mem.read_byte(echo), 0xCF);
    }

    #[test]
    fn unusable_region_reads_ff() {
        let mem = MemoryBus::new(make_empty_16kb_rom());
        // Underlying data is zero, but prohibited reads return 0xFF
        assert_eq!(mem.read_byte(0xFEA0), 0xFF);
        assert_eq!(mem.read_byte(0xFEFF), 0xFF);
    }

    #[test]
    fn unusable_region_write_is_ignored() {
        let mut mem = MemoryBus::new(make_empty_16kb_rom());
        // Write to Unusable; then underlying data stays at 0x00 (not 0x77)
        mem.write_byte(0xFEB0, 0x77);
        // If we bypass the Unusable barrier:
        assert_eq!(mem.data[0xFEB0], 0x00);
    }
}
