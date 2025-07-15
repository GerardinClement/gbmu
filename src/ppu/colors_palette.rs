#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Clone, Debug)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Color {
    pub fn to_rgb(self) -> [u8; 3] {
        match self {
            Color::White => [255, 255, 255],
            Color::LightGray => [192, 192, 192],
            Color::DarkGray => [96, 96, 96],
            Color::Black => [0, 0, 0],
        }
    }

    pub fn from_rgb(rgb: [u8; 3]) -> Self {
        match rgb {
            [255, 255, 255] => Color::White,
            [192, 192, 192] => Color::LightGray,
            [96, 96, 96] => Color::DarkGray,
            [0, 0, 0] => Color::Black,
            _ => unreachable!(),
        }
    }

    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => unreachable!(),
        }
    }
}
