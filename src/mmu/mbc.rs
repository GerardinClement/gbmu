use std::cmp::min;

const ONLY_ROM_SIZE: usize = 0xC000;
const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

pub trait Mbc: Default{
    fn new(rom_image: &[u8]) -> Result<Self, String> where Self: Sized;
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
    ram_banks: Vec<[u8; RAM_BANK_SIZE]>,
}

fn get_rom_bank_size(rom: &[u8]) -> Result<usize, String>{
    let code = rom[0x149];
    match code {
        0 => Ok(2),
        1 => Ok(4),
        2 => Ok(8),
        3 => Ok(16),
        4 => Ok(32),
        5 => Ok(64),
        6 => Ok(128),
        7 => Ok(256),
        8 => Ok(512),
        _ => Err(format!("Rom size code can't be {}", code))
    }
}

fn get_ram_bank_size(rom: &[u8]) -> Result<usize, String>{
    let code = rom[0x148];
    match code {
        0 => Ok(0),
        1 => Ok(0),
        2 => Ok(1),
        3 => Ok(4),
        4 => Ok(16),
        5 => Ok(8),
        _ => Err(format!("Ram size code can't be {}", code)),
    }
}

fn map_rom_into_bank(rom_image: &[u8]) -> Result<Vec<[u8; ROM_BANK_SIZE]>, String> {
    let banks: Vec<[u8; ROM_BANK_SIZE]> = rom_image .chunks_exact(ROM_BANK_SIZE)
        .map(|slice|{
            let mut data = [0; ROM_BANK_SIZE];
            data.copy_from_slice(&slice);
            data
        }).collect();
    let supposed_rom_bank_size = get_rom_bank_size(rom_image)?;
    if banks.iter().count() != supposed_rom_bank_size {
        return Err(
            format!("Inconsistent Rom Header : size must be : {}", supposed_rom_bank_size)
        );
    }
    Ok(banks)
}

fn map_ram_banks(rom_image: &[u8]) -> Result<Vec<[u8; RAM_BANK_SIZE]>, String> {
    let supposed_ram_bank_size = get_ram_bank_size(rom_image)?;
    Ok(vec![[0u8; RAM_BANK_SIZE]; supposed_ram_bank_size])
}

impl Mbc for Mbc1 {
    fn new(rom_image: &[u8]) -> Result<Self, String> {
        let banks = map_rom_into_bank(rom_image)?;
        let ram_banks = map_ram_banks(rom_image)?;
        if ram_banks.iter().count() > 4 {
            Err(
                format!("Supposed ram bank size can't be more than 4 in mbc1 cartridge.")
            )
        } else {
            Ok(Mbc1 {
                banks,
                ram_gate_register: false,
                bank_register_1: 0b1,
                bank_register_2: 0b0,
                mode_register: false,
                ram_banks,
            })
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
                self.banks[
                    ((self.bank_register_2 << 5) + self.bank_register_1) as usize
                ][
                    addr as usize - ROM_BANK_SIZE as usize
                ]
            },
            0xA000..0xC000 => {
                if self.ram_gate_register {
                    self.ram_banks[
                        self.mode_register as usize * self.bank_register_2 as usize
                    ][addr as usize - 0xA000]
                } else {
                    0
                }
            },
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..0x2000 => self.ram_gate_register = (val & 0b1010) == 0b1010,
            0x2000..0x4000 => self.bank_register_1 = val & 0b11111,
            0x4000..0x6000 => self.bank_register_2 = val & 0b11,
            0x6000..0x8000 => self.mode_register = (val & 0b1) == 0b1,
            0xA000..0xC000 => {
                if self.ram_gate_register {
                self.ram_banks[
                        self.mode_register as usize * self.bank_register_2 as usize
                    ][addr as usize - 0xA000] = val
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Default)]
pub struct Mbc2 {
    rom_banks: Vec<[u8; ROM_BANK_SIZE]>
    ram_gate_register: bool,
    rom_bank_register: u8,
    ram_banks: Vec<[u8; RAM_BANK_SIZE]>,
}

impl Mbc for Mbc2 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..0x4000 => {
                self.rom_banks[0][addr as usize]
            },
            0x4000..0x8000 => {
                self.rom_banks[self.rom_bank_register as usize][(addr - 0x4000) as usize]
            },
            0xA000..0xC000 => {
                if self.ram_gate_register {
                    self.ram_banks[0][(addr & 0b1111_1111) as usize]
                } else {
                    0
                }
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..0x4000 => {
                if addr & 0b1_0000_0000 == 0b1_0000_0000 {
                    self.ram_gate_register = val & 0b1010 == 0b1010
                } else {
                    let new_value = val & 0b1111;
                    self.rom_bank_register = (new_value == 0) as u8 + new_value;
                }
            },
            0x4000..0x8000 => { }, // do nothing 
            0xA000..0xC000 => { 
                if self.ram_gate_register {
                    self.ram_banks[0][(addr & 0b1111_1111) as usize] = val;
                }

            },
            _ => unreachable!()
        }
    }

    fn new(rom_image: &[u8]) -> Result<Self, String> where Self: Sized {
        let rom_banks = map_rom_into_bank(rom_image)?;
        Ok(Mbc2{
            rom_banks,
            ram_gate_register: false,
            rom_bank_register: 0b0001,
            ram_banks: vec![[0; RAM_BANK_SIZE]; 1],
        })
    }
}

pub struct RomOnly {
    bank: [u8; ONLY_ROM_SIZE],
}

impl Default for RomOnly {
    fn default() -> Self {
        RomOnly {
            bank: [0; ONLY_ROM_SIZE] 
        }
    }
}

impl Mbc for RomOnly{
    fn new(rom_image: &[u8]) -> Result<Self, String> {
        let mut bank = [0; ONLY_ROM_SIZE];
        let end = min(ONLY_ROM_SIZE, rom_image.len());
        bank[..end].copy_from_slice(&rom_image[..end]);
        Ok(RomOnly {
            bank
        })
    }

    fn read(&self, addr: u16) -> u8 {
        self.bank[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        if (0xA000..0xC000).contains(&addr) {
            self.bank[addr as usize] = val
        }

    }
}

pub struct Mbc3 {

}

impl Mbc for Mbc3 {
    fn read(&self, addr: u16) -> u8 {
        0
    }
    fn write(&mut self, addr: u16, val: u8) {
        
    }
    fn new(rom_image: &[u8]) -> Result<Self, String> where Self: Sized {
        Ok(
            Mbc5 {

            }
        )
        
    }

}
