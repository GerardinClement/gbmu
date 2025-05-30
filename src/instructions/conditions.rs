pub enum COND {
	NZ = 0,
	Z = 1,
	NC = 2,
	C = 3
}


impl From<u8> for COND {
    fn from(value: u8) -> Self {
        match value {
            0 => COND::NZ,
            1 => COND::Z,
            2 => COND::NC,
            3 => COND::C,
            _ => panic!("Invalid value for COND: {}", value),
        }
    }
}


impl COND {
    pub fn test(&self, z: bool, c: bool) -> bool {
        match self {
            COND::NZ => !z,
            COND::Z  => z,
            COND::NC => !c,
            COND::C  => c,
        }
    }
}