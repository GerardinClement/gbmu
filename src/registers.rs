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
	fn get_af(&self) -> u16 {
		let byte: u8 = u8::from(self.f.clone());
		return (self.a as u16) << 8 | byte as u16;
	}

	fn set_af(&mut self, value: u16) {
		let flags: FlagsRegister = FlagsRegister::from((value & 0xFF) as u8);
		self.a = ((value & 0xFF00) >> 8) as u8;
		self.f = flags;
	}

	fn get_bc(&self) -> u16 {
		return (self.b as u16) << 8 | self.c as u16;
	}

	fn set_bc(&mut self, value: u16) {
		self.b = ((value & 0xFF00) >> 8) as u8;
		self.c = (value & 0xFF) as u8;
	}

	fn get_de(&self) -> u16 {
		return (self.d as u16) << 8 | self.e as u16;
	}

	fn set_de(&mut self, value: u16) {
		self.d = ((value & 0xFF00) >> 8) as u8;
		self.e = (value & 0xFF) as u8;
	}

	fn get_hl(&self) -> u16 {
		return (self.h as u16) << 8 | self.l as u16;
	}

	fn set_hl(&mut self, value: u16) {
		self.h = ((value & 0xFF00) >> 8) as u8;
		self.l = (value & 0xFF) as u8;
	}
}