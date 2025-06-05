#![allow(unused_variables)]
#![allow(dead_code)]


pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Self {
        Memory { data: [0; 0x10000] }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        self.data[addr as usize] = val;
    }
}

#[test]
fn basic_ram_read_write() {
    let mut mem = Memory::new();
    mem.write_byte(0xC000, 0x42);
    assert_eq!(mem.read_byte(0xC000), 0x42);
}
