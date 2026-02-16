use std::cmp::min;

const ROM_BANK_SIZE: usize = 0x4000;

pub trait Mbc {
    fn new(rom_image: &[u8]) -> Self where Self: Sized;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[derive(Clone)]
pub struct RomOnly {
    banks: Vec<[u8; 0x4000]>,
    current: usize,
}

impl Mbc for RomOnly{
    fn new(rom_image: &[u8]) -> Self {
        let mut banks = Vec::new();
        let mut offset = 0;

        while offset < rom_image.len() {
            let mut bank = [0u8; ROM_BANK_SIZE];

            let end = min(offset + ROM_BANK_SIZE, rom_image.len());
            let len = end - offset;

            bank[..len].copy_from_slice(&rom_image[offset..end]);
            banks.push(bank);
            offset += ROM_BANK_SIZE;
        }
        if banks.len() < 2 {
            banks.resize(2, [0u8; ROM_BANK_SIZE]);
        }

        RomOnly { banks, current: 1 }
    }

    fn read(&self, addr: u16) -> u8 {
        let i = addr as usize;

        if i < ROM_BANK_SIZE {
            self.banks[0][i]
        } else {
            let off = i - ROM_BANK_SIZE;

            self.banks[self.current][off]
        }
    }

    fn write(&mut self, addr: u16, val: u8) {}
}

impl RomOnly {
    fn bank_count(&self) -> usize {
        self.banks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::RomOnly;

    #[test]
    fn small_rom_creates_two_banks() {
        let data = vec![0xAA; 100]; // 100 bytes
        let mbc = RomOnly::new(&data);

        // Should have at least two banks
        assert_eq!(mbc.banks.len(), 2);
        // First bank begins with your data...
        for i in 0..100 {
            assert_eq!(mbc.banks[0][i], 0xAA);
        }
        // ...and the rest of bank 0 is zero
        for i in 100..0x4000 {
            assert_eq!(mbc.banks[0][i], 0);
        }
        // Bank 1 is entirely zeros
        assert!(mbc.banks[1].iter().all(|&b| b == 0));
    }

    #[test]
    fn multi_bank_rom_splits_correctly() {
        // Create 20 KiB of incrementing bytes: 0,1,2,…,19999
        let data: Vec<u8> = (0..20_480).map(|i| (i % 256) as u8).collect();
        let mbc = RomOnly::new(&data);

        assert_eq!(mbc.banks.len(), 2);
        // First bank matches bytes 0..16384
        for i in 0..0x4000 {
            assert_eq!(mbc.banks[0][i], (i % 256) as u8);
        }
        // Second bank matches bytes 16384..20480
        for i in 0..(20_480 - 0x4000) {
            assert_eq!(mbc.banks[1][i], ((i + 0x4000) % 256) as u8);
        }
    }
}
