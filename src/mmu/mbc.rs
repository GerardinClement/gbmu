use std::cmp::{min};

#[derive (Clone)]
pub struct Mbc {
    banks: Vec<[u8; 0x4000]>,
    current: usize,
}

impl Mbc {
    pub fn new(rom_image: &[u8]) -> Self {
        let mut banks = Vec::new();
        let mut offset = 0;

        while offset < rom_image.len() {
            let mut bank = [0u8; 0x4000];

            let end = min(offset + 0x4000, rom_image.len());
            let len = end - offset;
            
            bank[..len].copy_from_slice(&rom_image[offset..end]);
            banks.push(bank);
            offset += 0x4000;
        }
        if banks.len() < 2 {
            banks.resize(2, [0u8; 0x4000]);
        }

        Mbc { banks, current: 1 }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let i = addr as usize;

        if i < 0x4000 {
            self.banks[0][i]
        } else {
            let off = i - 0x4000;

            self.banks[self.current][off]
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {}
}