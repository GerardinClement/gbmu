#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PpuMode {
    #[default]
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

#[derive(Default)]
pub struct LcdStatus {
    lyc_int_select: bool,
    mode_2_int_select: bool,
    mode_1_int_select: bool,
    mode_0_int_select: bool,
    lyc_equals_ly: bool,
    ppu_mode: PpuMode,
}

impl LcdStatus {
    pub fn new() -> Self {
        LcdStatus {
            lyc_int_select: false,
            mode_2_int_select: false,
            mode_1_int_select: false,
            mode_0_int_select: false,
            lyc_equals_ly: false,
            ppu_mode: PpuMode::HBlank,
        }
    }

    pub fn get_ppu_mode(&self) -> PpuMode {
        self.ppu_mode
    }

    pub fn update_ppu_mode(&mut self, mode: PpuMode) {
        self.ppu_mode = mode;
    }

    pub fn set_lyc_equals_ly(&mut self, equals: bool) {
        self.lyc_equals_ly = equals;
    }
}
