use std::cmp::min;

const ONLY_ROM_SIZE: usize = 0x8000;
const ROM_BANK_SIZE: usize = 0x4000;

pub trait Mbc: Default{
    fn new(rom_image: &[u8]) -> Self where Self: Sized;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[derive(Clone, Default)]
pub  struct Mbc1 {
    banks: Vec<[u8; ROM_BANK_SIZE]>,
    ram_gate_register: bool, // If ramg is set to 0b1010 -> 
    bank_register_1: u8,
    bank_register_2: u8,
    mode_register: bool,
}

impl Mbc for Mbc1 {
    fn new(rom_image: &[u8]) -> Self {
        let chunks: Vec<[u8; ROM_BANK_SIZE]>= rom_image
            .chunks_exact(ROM_BANK_SIZE)
            .map(|slice|{
                let mut data = [0; ROM_BANK_SIZE];
                data.copy_from_slice(&slice);
                data
            }).collect();
        Mbc1 {
            banks: chunks,
            ram_gate_register: false,
            bank_register_1: 0b1,
            bank_register_2: 0b0,
            mode_register: false,

        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..0x4000 => {
                if self.mode_register {
                    self.banks[0][addr as usize]
                } else {
                    self.banks[(self.bank_register_2 << 5) as usize][addr as usize]
                }
            },
            0x4000..0x8000 => {
                self.banks[((self.bank_register_2 << 5) + self.bank_register_1) as usize][addr as usize - ROM_BANK_SIZE as usize]
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..0x2000 => self.ram_gate_register = (val & 0b1010) == 0b1010,
            0x2000..0x4000 => self.bank_register_1 = val & 0b11111,
            0x4000..0x6000 => self.bank_register_2 = val & 0b11,
            0x6000..0x8000 => self.mode_register = (val & 0b1) == 0b1,
            _ => unreachable!()
        }
    }
}

#[derive(Clone)]
pub struct RomOnly {
    bank: [u8; 0x8000],
}

impl Default for RomOnly {
    fn default() -> Self {
        RomOnly {
            bank: [0; 0x8000] 
        }
    }
}

impl Mbc for RomOnly{
    fn new(rom_image: &[u8]) -> Self {
        let mut bank = [0u8; ONLY_ROM_SIZE];
        let end = min(ONLY_ROM_SIZE, rom_image.len());
        bank[..end].copy_from_slice(&rom_image[..end]);
        RomOnly {
            bank
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bank[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {}
}
