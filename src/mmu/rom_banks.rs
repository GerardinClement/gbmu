pub struct RomBanks {
    banks: Vec<[u8; 0x4000]>,
    current: usize,
}

impl RomBanks {
    pub fn new(rom_image: &[u8]) -> Self {

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

    pub fn write(&mut self, addr: u16, val: u8) {
        
    }
}