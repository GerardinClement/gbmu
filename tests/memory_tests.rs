use gbmu::memory::Memory;


#[cfg(test)]
mod tests {
    #[test]
    fn basic_ram_read_write() {
        let mut mem = Memory::new();
        mem.write_byte(0xC000, 0x42);
        assert_eq!(mem.read_byte(0xC000), 0x42);
    }

    #[test]
    fn vram_read_write() {
        let mut mem = Memory::new();
        mem.write_byte(0x8100, 0x42);
        assert_eq!(mem.read_byte(0x8100), 0x42);
    }

    #[test]
    fn io_registers_read_write() {
        let mut mem = Memory::new();
        let io1 = 0xFF00;
        let io2 = 0xFF7F;
        mem.write_byte(io1, 0x11);
        mem.write_byte(io2, 0x22);
        assert_eq!(mem.read_byte(io1), 0x11);
        assert_eq!(mem.read_byte(io2), 0x22);
    }

    #[test]
    fn rom_is_read_only() {
        let mut mem = Memory::new();
        let rom_addr = 0x2000;
        mem.write_byte(rom_addr, 0x99);
        assert_eq!(mem.read_byte(rom_addr), 0x00);
    }

    #[test]
    fn echo_ram_mirrors_wram() {
        let mut mem = Memory::new();
        let echo = 0xE456;
        let real = 0xC456;
        mem.write_byte(echo, 0xCD);
        assert_eq!(mem.read_byte(real), 0xCD);
    }

    #[test]
    fn echo_ram_mirrors_backwards() {
        let mut mem = Memory::new();
        let echo = 0xE567;
        let real = 0xC567;
        mem.write_byte(real, 0xCF);
        assert_eq!(mem.read_byte(echo), 0xCF);
    }

    #[test]
    fn unusable_region_reads_ff() {
        let mem = Memory::new();
        // Underlying data is zero, but prohibited reads return 0xFF
        assert_eq!(mem.read_byte(0xFEA0), 0xFF);
        assert_eq!(mem.read_byte(0xFEFF), 0xFF);
    }

    #[test]
    fn unusable_region_write_is_ignored() {
        let mut mem = Memory::new();
        // Write to Unusable; then underlying data stays at 0x00 (not 0x77)
        mem.write_byte(0xFEB0, 0x77);
        // If we bypass the Unusable barrier:
        assert_eq!(mem.data[0xFEB0], 0x00);
    }
}