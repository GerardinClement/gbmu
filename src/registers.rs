use crate::flags_registers::FlagsRegister;

pub struct Registers {
	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	f: FlagsRegister,
	l: u8,
}

impl Registers {
	pub fn set_r8_value(&mut self, target: u8, value: u8) {
		match target {
			0 => self.b = value,
			1 => self.c = value,
			2 => self.d = value,
			3 => self.e = value,
			4 => self.h = value,
			5 => self.l = value,
			6 => self.l = value, //replace with [HL] when implemented
			7 => self.a = value,
			_ => panic!("Invalid register index: {}", target),
		}
	}

	pub fn set_r16_value(&mut self, target: u8, value: u16) -> u16 {
		match target {
			0 => self.set_bc(value),
			1 => self.set_de(value),
			2 => self.set_hl(value),
			3 => self.set_hl(value), //replace with SP when implemented
			_ => panic!("Invalid 16-bit register index: {}", target),
		}
		return value;
	}

	pub fn get_a(&self) -> u8 {
		return self.a.clone();
	}

	pub fn set_a(&mut self, value: u8) {
		self.a = value;
	}

	pub fn get_b(&self) -> u8 {
		return self.b.clone();
	}

	pub fn set_b(&mut self, value: u8) {
		self.b = value;
	}

	pub fn get_c(&self) -> u8 {
		return self.c.clone();
	}

	pub fn set_c(&mut self, value: u8) {
		self.c = value;
	}

	pub fn get_d(&self) -> u8 {
		return self.d.clone();
	}

	pub fn set_d(&mut self, value: u8) {
		self.d = value;
	}

	pub fn get_e(&self) -> u8 {
		return self.e.clone();
	}

	pub fn set_e(&mut self, value: u8) {
		self.e = value;
	}

	pub fn get_h(&self) -> u8 {
		return self.h.clone();
	}

	pub fn set_h(&mut self, value: u8) {
		self.h = value;
	}

	pub fn get_l(&self) -> u8 {
		return self.l.clone();
	}

	pub fn set_l(&mut self, value: u8) {
		self.l = value;
	}

	pub fn get_f(&self) -> FlagsRegister {
		return self.f.clone();
	}

	pub fn set_f(&mut self, flags: FlagsRegister) {
		self.f = flags;
	}

	pub fn get_af(&self) -> u16 {
		let byte: u8 = u8::from(self.f.clone());
		return (self.a as u16) << 8 | byte as u16;
	}

	pub fn set_af(&mut self, value: u16) {
		let flags: FlagsRegister = FlagsRegister::from((value & 0xFF) as u8);
		self.a = ((value & 0xFF00) >> 8) as u8;
		self.f = flags;
	}

	pub fn get_bc(&self) -> u16 {
		return (self.b as u16) << 8 | self.c as u16;
	}

	pub fn set_bc(&mut self, value: u16) {
		self.b = ((value & 0xFF00) >> 8) as u8;
		self.c = (value & 0xFF) as u8;
	}

	pub fn get_de(&self) -> u16 {
		return (self.d as u16) << 8 | self.e as u16;
	}

	pub fn set_de(&mut self, value: u16) {
		self.d = ((value & 0xFF00) >> 8) as u8;
		self.e = (value & 0xFF) as u8;
	}

	pub fn get_hl(&self) -> u16 {
		return (self.h as u16) << 8 | self.l as u16;
	}

	pub fn set_hl(&mut self, value: u16) {
		self.h = ((value & 0xFF00) >> 8) as u8;
		self.l = (value & 0xFF) as u8;
	}
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: FlagsRegister::default(),
        }
    }
}