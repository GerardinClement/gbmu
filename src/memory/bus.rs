use crate::memory::region::MemoryRegion;

pub struct MemoryBus {
    data: [u8; 0x10000], // 10000 = 65536 = 0xFFFF
}

impl MemoryBus {
    pub fn new() -> Self {
        MemoryBus { 
            data: [0; 0x10000],
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let region = MemoryRegion::from(addr);

        if !region.is_readable() {
            return 0xFF;
        }

        let phys = region.translate_into_physical_address(addr);

        self.data[phys as usize]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) { // mut means that we have a mutable borrow, writes will change data.
        let region = MemoryRegion::from(addr);
    
        if !region.is_writable() {
            return ;
        }

        let phys = region.translate_into_physical_address(addr);

        self.data[phys as usize] = val;
    }
}


#[cfg(test)]
mod tests {
    use super::MemoryBus;

    #[test]
    fn basic_ram_read_write() {
        let mut mem = MemoryBus::new();
        mem.write_byte(0xC000, 0x42);
        assert_eq!(mem.read_byte(0xC000), 0x42);
    }

    #[test]
    fn vram_read_write() {
        let mut mem = MemoryBus::new();
        mem.write_byte(0x8100, 0x42);
        assert_eq!(mem.read_byte(0x8100), 0x42);
    }

    #[test]
    fn io_registers_read_write() {
        let mut mem = MemoryBus::new();
        let io1 = 0xFF00;
        let io2 = 0xFF7F;
        mem.write_byte(io1, 0x11);
        mem.write_byte(io2, 0x22);
        assert_eq!(mem.read_byte(io1), 0x11);
        assert_eq!(mem.read_byte(io2), 0x22);
    }

    #[test]
    fn rom_is_read_only() {
        let mut mem = MemoryBus::new();
        let rom_addr = 0x2000;
        mem.write_byte(rom_addr, 0x99);
        assert_eq!(mem.read_byte(rom_addr), 0x00);
    }

    #[test]
    fn echo_ram_mirrors_wram() {
        let mut mem = MemoryBus::new();
        let echo = 0xE456;
        let real = 0xC456;
        mem.write_byte(echo, 0xCD);
        assert_eq!(mem.read_byte(real), 0xCD);
    }

    #[test]
    fn echo_ram_mirrors_backwards() {
        let mut mem = MemoryBus::new();
        let echo = 0xE567;
        let real = 0xC567;
        mem.write_byte(real, 0xCF);
        assert_eq!(mem.read_byte(echo), 0xCF);
    }

    #[test]
    fn unusable_region_reads_ff() {
        let mem = MemoryBus::new();
        // Underlying data is zero, but prohibited reads return 0xFF
        assert_eq!(mem.read_byte(0xFEA0), 0xFF);
        assert_eq!(mem.read_byte(0xFEFF), 0xFF);
    }

    #[test]
    fn unusable_region_write_is_ignored() {
        let mut mem = MemoryBus::new();
        // Write to Unusable; then underlying data stays at 0x00 (not 0x77)
        mem.write_byte(0xFEB0, 0x77);
        // If we bypass the Unusable barrier:
        assert_eq!(mem.data[0xFEB0], 0x00);
    }
}