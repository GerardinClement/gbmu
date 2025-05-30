use crate::{cpu::MemoryBus, flags_registers::FlagsRegister};


#[repr(u8)]
pub enum R8 {
	B = 0,
	C = 1,
	D = 2,
	E = 3,
	H = 4,
	L = 5,
	HLIndirect = 6,
	A = 7,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum R16 {
	BC = 0,
	DE = 1,
	HL = 2,
	SP = 3, // To be implemented
}

impl From<u8> for R16 {
	fn from(value: u8) -> Self {
		match value {
			0 => R16::BC,
			1 => R16::DE,
			2 => R16::HL,
			3 => R16::SP,
			_ => panic!("Invalid value for R16: {}", value),
		}
	}
}

pub struct Registers {
    r8: [u8; 8],
    f: FlagsRegister,
}


impl Registers {
    pub fn set_r8_value(&mut self, target: R8, value: u8) {
        self.r8[target as usize] = value;
    }

    pub fn get_r8_value(&self, target: R8) -> u8 {
        self.r8[target as usize]
    }

    pub fn get_r16_value(&self, target: R16) -> u16 {
        match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            R16::SP => self.get_hl(), // replace with SP when implemented
        }
    }

    pub fn set_r16_value(&mut self, target: R16, value: u16) {
        match target {
            R16::BC => self.set_bc(value),
            R16::DE => self.set_de(value),
            R16::HL => self.set_hl(value),
            R16::SP => self.set_hl(value), // replace with SP when implemented
        }
    }

    pub fn set_r16_mem_value(&mut self, memory: &mut MemoryBus, target: R16, value: u8) {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            R16::SP => self.get_hl(), // replace with SP when implemented
        };
        memory.write_byte(addr, value);
    }

    pub fn get_r16_mem_value(&self, memory: &MemoryBus, target: R16) -> u8 {
        let addr = match target {
            R16::BC => self.get_bc(),
            R16::DE => self.get_de(),
            R16::HL => self.get_hl(),
            R16::SP => self.get_hl(), // replace with SP when implemented
        };
        memory.read_byte(addr)
    }

    pub fn add_to_r16(&mut self, target: R16, value: u16) {
        let r16_value = self.get_r16_value(target);
        let (new_value, did_overflow) = r16_value.overflowing_add(value);
        
        self.set_r16_value(target, new_value);
        self.f.zero = value == 0;
        self.f.subtract = false;
        self.f.carry = did_overflow;
        self.f.half_carry = (r16_value & 0xFFF) + (value & 0xFFF) > 0xFFF;
    }

	pub fn get_a(&self) -> u8 {
		self.r8[R8::A as usize]
	}

	pub fn get_b(&self) -> u8 {
		self.r8[R8::B as usize]
	}

	pub fn get_c(&self) -> u8 {
		self.r8[R8::C as usize]
	}

	pub fn get_d(&self) -> u8 {
		self.r8[R8::D as usize]
	}

	pub fn get_e(&self) -> u8 {
		self.r8[R8::E as usize]
	}

	pub fn get_h(&self) -> u8 {
		self.r8[R8::H as usize]
	}

	pub fn get_l(&self) -> u8 {
		self.r8[R8::L as usize]
	}

    pub fn get_af(&self) -> u16 {
        let byte: u8 = u8::from(self.f.clone());
        ((self.r8[R8::A as usize] as u16) << 8) | (byte as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.r8[R8::A as usize] = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xFF) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        ((self.r8[R8::B as usize] as u16) << 8) | (self.r8[R8::C as usize] as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.r8[R8::B as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::C as usize] = (value & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.r8[R8::D as usize] as u16) << 8) | (self.r8[R8::E as usize] as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.r8[R8::D as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::E as usize] = (value & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.r8[R8::H as usize] as u16) << 8) | (self.r8[R8::L as usize] as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.r8[R8::H as usize] = ((value & 0xFF00) >> 8) as u8;
        self.r8[R8::L as usize] = (value & 0xFF) as u8;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            r8: [0; 8],
            f: FlagsRegister::default(),
        }
    }
}
