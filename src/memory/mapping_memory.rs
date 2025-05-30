
/*
- read bytes and write bytes
- write bytes check if the memory isn't the rom region, if it is it doesn't write.
- read and write check if it's echo ram and copi work ram bank instead.

*/

#[derive(PartialEq, Eq)]
enum MemoryRegion {
    Rom, // 0x000-0x7FFF: read-only
    Echo, // 0xE000-0xFDFF: mirror of C000-DDFF
    Ram, // Everything else (for now)
}

impl MemoryRegion {
    // Given any address, pick its region.
    fn from(addr: u16) -> Self {
        match addr {
            0x0000..=0x7FFF => MemoryRegion::Rom,
            0xE000..=0xFDFF => MemoryRegion::Echo,
            _ => MemoryRegion::Ram,
        }
    }

    fn is_writable(&self) -> bool {
        !matches!(self, MemoryRegion::Rom)
    }

    fn translate_in_physical_address(&self, addr: u16) -> u16 {
        match self {
            MemoryRegion::Echo => addr - 0x2000,
            _ => addr,
        }
    }
}

pub struct Memory {
    data: [u8; 0x10000], // 10000 = 65536 = 0xFFFF
}

impl Memory {
    pub fn new() -> Self {
        Memory { 
            data: [0; 0x10000],
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let region = MemoryRegion::from(addr);

        let phys = region.translate_in_physical_address(addr);

        self.data[phys as usize]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) { // mut means that we have a mutable borrow, writes will change data.
        let region = MemoryRegion::from(addr);
    
        if !region.is_writable() {
            return ;
        }

        let phys = region.translate_in_physical_address(addr);

        self.data[phys as usize] = val;
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;

    #[test]
    fn basic_ram_read_write() {
        let mut mem = Memory::new();

        mem.write_byte(0xC000, 0x42);
        assert_eq!(mem.read_byte(0xC000), 0x42);
    }

    #[test]
    fn rom_is_read_only() {
        let mut mem = Memory::new();

        let mut addr: u16 = 0x2000;
        let mut val: u8 = 0x99;

        // Ensure ROM starts zeroed
        assert_eq!(mem.read_byte(addr), 0x00);

        // Attempt to write 0x99 into ROM
        mem.write_byte(addr, val);

        // It should still read back 0x00
        assert_eq!(mem.read_byte(addr), 0x00);

        
        // As a control writes outside ROM should stick:
        addr = 0xC000;
        val = 0x77;

        mem.write_byte(addr, val);
        assert_eq!(mem.read_byte(addr), val);
    }

    #[test]
    fn echo_ram_mirrors_wram() {
        let mut mem = Memory::new();

        let echo_addr: u16 = 0xE456;
        let wram_addr: u16 = 0xC456;
        let val: u8 = 0xCD;

        mem.write_byte(echo_addr, val);

        assert_eq!(mem.read_byte(wram_addr), val)
    }

    #[test]
    fn echo_ram_mirrors_wram_backward() {
        let mut mem = Memory::new();

        let echo_addr: u16 = 0xE567;
        let wram_addr: u16 = 0xC567;
        let val: u8 = 0xCF;

        mem.write_byte(wram_addr, val);

        assert_eq!(mem.read_byte(echo_addr), val)
    }
}
