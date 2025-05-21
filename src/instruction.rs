pub enum ArithmeticTarget {
	A, B, C, D, E, H, L,
}

pub enum JumpTest {
	None,
	Zero,
	NotZero,
	Carry,
	NotCarry,
}

pub enum JumpTarget {
	A16,
	HL,
}

pub enum LoadSource {
	A, B, C, D, E, H, L, BC, BCAddr, DE, DEAddr, HL, HLAddr, D8, D16, HLI, HLD
}

pub enum LoadTarget {
	A, B, C, D, E, H, L, BC, BCAddr, DE, DEAddr, HL, HLAddr, N16, HLI, HLD
}

pub enum Is16BitLoad {
	True,
	False
}

pub enum Instructions {
	ADD(ArithmeticTarget),
	JP(JumpTest, JumpTarget),
	LD(LoadTarget, LoadSource)
}

impl Instructions {
	pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instructions> {
		if prefixed {
			Instructions::from_byte_prefixed(byte)
		} else {
			Instructions::from_byte_not_prefixed(byte)
		}
	}

	fn from_byte_prefixed(byte: u8) -> Option<Instructions> {
		match byte {
			_ => panic!("Prefixed instructions {} not set", byte)
		}
	}

	fn from_byte_not_prefixed(byte: u8) -> Option<Instructions> {
		match byte {
			0x02 => Some(Instructions::LD(LoadTarget::BC, LoadSource::A)),
			0x06 => Some(Instructions::LD(LoadTarget::B, LoadSource::D8)),
			0x0A => Some(Instructions::LD(LoadTarget::A, LoadSource::BC)),
			0x0E => Some(Instructions::LD(LoadTarget::C, LoadSource::D8)),
			0x12 => Some(Instructions::LD(LoadTarget::DE, LoadSource::A)),
			0x16 => Some(Instructions::LD(LoadTarget::D, LoadSource::D8)),
			0x1A => Some(Instructions::LD(LoadTarget::A, LoadSource::DE)),
			0x1E => Some(Instructions::LD(LoadTarget::E, LoadSource::D8)),
			0x22 => Some(Instructions::LD(LoadTarget::HLI, LoadSource::A)),
			0x26 => Some(Instructions::LD(LoadTarget::H, LoadSource::D8)),
			0x2A => Some(Instructions::LD(LoadTarget::A, LoadSource::HLI)),
			0x2E => Some(Instructions::LD(LoadTarget::L, LoadSource::D8)),
			0x32 => Some(Instructions::LD(LoadTarget::HLD, LoadSource::A)),
			0x36 => Some(Instructions::LD(LoadTarget::HL, LoadSource::D8)),
			0x3A => Some(Instructions::LD(LoadTarget::HLD, LoadSource::A)),
			0x3E => Some(Instructions::LD(LoadTarget::A, LoadSource::D8)),
			// LD B, r
			0x40 => Some(Instructions::LD(LoadTarget::B, LoadSource::B)),
			0x41 => Some(Instructions::LD(LoadTarget::B, LoadSource::C)),
			0x42 => Some(Instructions::LD(LoadTarget::B, LoadSource::D)),
			0x43 => Some(Instructions::LD(LoadTarget::B, LoadSource::E)),
			0x44 => Some(Instructions::LD(LoadTarget::B, LoadSource::H)),
			0x45 => Some(Instructions::LD(LoadTarget::B, LoadSource::L)),
			0x46 => Some(Instructions::LD(LoadTarget::B, LoadSource::HL)),
			0x47 => Some(Instructions::LD(LoadTarget::B, LoadSource::A)),

			// LD C, r
			0x48 => Some(Instructions::LD(LoadTarget::C, LoadSource::B)),
			0x49 => Some(Instructions::LD(LoadTarget::C, LoadSource::C)),
			0x4A => Some(Instructions::LD(LoadTarget::C, LoadSource::D)),
			0x4B => Some(Instructions::LD(LoadTarget::C, LoadSource::E)),
			0x4C => Some(Instructions::LD(LoadTarget::C, LoadSource::H)),
			0x4D => Some(Instructions::LD(LoadTarget::C, LoadSource::L)),
			0x4E => Some(Instructions::LD(LoadTarget::C, LoadSource::HL)),
			0x4F => Some(Instructions::LD(LoadTarget::C, LoadSource::A)),

			// LD D, r
			0x50 => Some(Instructions::LD(LoadTarget::D, LoadSource::B)),
			0x51 => Some(Instructions::LD(LoadTarget::D, LoadSource::C)),
			0x52 => Some(Instructions::LD(LoadTarget::D, LoadSource::D)),
			0x53 => Some(Instructions::LD(LoadTarget::D, LoadSource::E)),
			0x54 => Some(Instructions::LD(LoadTarget::D, LoadSource::H)),
			0x55 => Some(Instructions::LD(LoadTarget::D, LoadSource::L)),
			0x56 => Some(Instructions::LD(LoadTarget::D, LoadSource::HL)),
			0x57 => Some(Instructions::LD(LoadTarget::D, LoadSource::A)),

			// LD E, r
			0x58 => Some(Instructions::LD(LoadTarget::E, LoadSource::B)),
			0x59 => Some(Instructions::LD(LoadTarget::E, LoadSource::C)),
			0x5A => Some(Instructions::LD(LoadTarget::E, LoadSource::D)),
			0x5B => Some(Instructions::LD(LoadTarget::E, LoadSource::E)),
			0x5C => Some(Instructions::LD(LoadTarget::E, LoadSource::H)),
			0x5D => Some(Instructions::LD(LoadTarget::E, LoadSource::L)),
			0x5E => Some(Instructions::LD(LoadTarget::E, LoadSource::HL)),
			0x5F => Some(Instructions::LD(LoadTarget::E, LoadSource::A)),

			// LD H, r
			0x60 => Some(Instructions::LD(LoadTarget::H, LoadSource::B)),
			0x61 => Some(Instructions::LD(LoadTarget::H, LoadSource::C)),
			0x62 => Some(Instructions::LD(LoadTarget::H, LoadSource::D)),
			0x63 => Some(Instructions::LD(LoadTarget::H, LoadSource::E)),
			0x64 => Some(Instructions::LD(LoadTarget::H, LoadSource::H)),
			0x65 => Some(Instructions::LD(LoadTarget::H, LoadSource::L)),
			0x66 => Some(Instructions::LD(LoadTarget::H, LoadSource::HL)),
			0x67 => Some(Instructions::LD(LoadTarget::H, LoadSource::A)),

			// LD L, r
			0x68 => Some(Instructions::LD(LoadTarget::L, LoadSource::B)),
			0x69 => Some(Instructions::LD(LoadTarget::L, LoadSource::C)),
			0x6A => Some(Instructions::LD(LoadTarget::L, LoadSource::D)),
			0x6B => Some(Instructions::LD(LoadTarget::L, LoadSource::E)),
			0x6C => Some(Instructions::LD(LoadTarget::L, LoadSource::H)),
			0x6D => Some(Instructions::LD(LoadTarget::L, LoadSource::L)),
			0x6E => Some(Instructions::LD(LoadTarget::L, LoadSource::HL)),
			0x6F => Some(Instructions::LD(LoadTarget::L, LoadSource::A)),

			// LD (HL), r
			0x70 => Some(Instructions::LD(LoadTarget::HL, LoadSource::B)),
			0x71 => Some(Instructions::LD(LoadTarget::HL, LoadSource::C)),
			0x72 => Some(Instructions::LD(LoadTarget::HL, LoadSource::D)),
			0x73 => Some(Instructions::LD(LoadTarget::HL, LoadSource::E)),
			0x74 => Some(Instructions::LD(LoadTarget::HL, LoadSource::H)),
			0x75 => Some(Instructions::LD(LoadTarget::HL, LoadSource::L)),
			0x77 => Some(Instructions::LD(LoadTarget::HL, LoadSource::A)),

			// LD A, r
			0x78 => Some(Instructions::LD(LoadTarget::A, LoadSource::B)),
			0x79 => Some(Instructions::LD(LoadTarget::A, LoadSource::C)),
			0x7A => Some(Instructions::LD(LoadTarget::A, LoadSource::D)),
			0x7B => Some(Instructions::LD(LoadTarget::A, LoadSource::E)),
			0x7C => Some(Instructions::LD(LoadTarget::A, LoadSource::H)),
			0x7D => Some(Instructions::LD(LoadTarget::A, LoadSource::L)),
			0x7E => Some(Instructions::LD(LoadTarget::A, LoadSource::HL)),
			0x7F => Some(Instructions::LD(LoadTarget::A, LoadSource::A)),

			0x80 => Some(Instructions::ADD(ArithmeticTarget::B)),
			0xE9 => Some(Instructions::JP(JumpTest::None, JumpTarget::HL)),

			_ => panic!("Not prefixed instructions {} not set", byte)
		}
	}
}
