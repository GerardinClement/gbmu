#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Default)]
pub struct LcdStatus {
    lyc_int_select: bool,
    mode_2_int_select: bool,
    mode_1_int_select: bool,
    mode_0_int_select: bool,
    lyc_equals_ly: bool,
    ppu_mode: u8, // 0: H-Blank, 1: V-Blank, 2: OAM Search, 3: Pixel Transfer
}

impl LcdStatus {
    pub fn new() -> Self {
        LcdStatus {
            lyc_int_select: false,
            mode_2_int_select: false,
            mode_1_int_select: false,
            mode_0_int_select: false,
            lyc_equals_ly: false,
            ppu_mode: 0, // Default to H-Blank
        }
    }

    pub fn update_ppu_mode(&mut self, mode: u8) {
        self.ppu_mode = mode;
    }

    pub fn set_lyc_equals_ly(&mut self, equals: bool) {
        self.lyc_equals_ly = equals;
    }
}
