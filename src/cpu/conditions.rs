#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug)]
pub enum Cond {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
    None = 4,
}

impl From<u8> for Cond {
    fn from(value: u8) -> Self {
        match value {
            0 => Cond::NZ,
            1 => Cond::Z,
            2 => Cond::NC,
            3 => Cond::C,
            _ => Cond::None,
        }
    }
}

impl Cond {
    pub fn test(&self, z: bool, c: bool) -> bool {
        match self {
            Cond::NZ => !z,
            Cond::Z => z,
            Cond::NC => !c,
            Cond::C => c,
            Cond::None => true,
        }
    }
}
