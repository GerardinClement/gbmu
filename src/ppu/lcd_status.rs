#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    #[default]
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
            ppu_mode: PpuMode::OamSearch,
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

    pub fn get_lyc_equals_ly(&self) -> bool {
        self.lyc_equals_ly
    }

    pub fn struct_to_byte(&self) -> u8 {
        let mut stat = 0u8;

        // Bit 6: LYC interrupt enable
        if self.lyc_int_select {
            stat |= 0b0100_0000;
        }

        // Bit 5: Mode 2 interrupt enable
        if self.mode_2_int_select {
            stat |= 0b0010_0000;
        }

        // Bit 4: Mode 1 interrupt enable
        if self.mode_1_int_select {
            stat |= 0b0001_0000;
        }

        // Bit 3: Mode 0 interrupt enable
        if self.mode_0_int_select {
            stat |= 0b0000_1000;
        }

        // Bit 2: LYC == LY flag (read-only)
        if self.lyc_equals_ly {
            stat |= 0b0000_0100;
        }

        // Bits 1-0: PPU mode
        stat |= self.ppu_mode as u8;

        stat
    }

    pub fn update_from_byte(&mut self, value: u8) {
        self.lyc_int_select = (value & 0b0100_0000) != 0;
        self.mode_2_int_select = (value & 0b0010_0000) != 0;
        self.mode_1_int_select = (value & 0b0001_0000) != 0;
        self.mode_0_int_select = (value & 0b0000_1000) != 0;

        // Bits 2, 1, 0 are read-only
    }
}
