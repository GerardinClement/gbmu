#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Range;

pub const TILE_MAP_0: Range<u16> = 0x9800..0x9BFF;
pub const TILE_MAP_1: Range<u16> = 0x9C00..0x9FFF;

pub const TILE_DATA_0: Range<u16> = 0x8800..0x97FF;
pub const TILE_DATA_1: Range<u16> = 0x8000..0x8FFF;

const PPU_ENABLE_MASK: u8 = 0b1000_0000;
const WINDOW_TILE_MAP_MASK: u8 = 0b0100_0000;
const WINDOW_ENABLE_MASK: u8 = 0b0010_0000;
const BG_WINDOW_TILE_DATA_MASK: u8 = 0b0001_0000;
const BG_TILE_MAP_MASK: u8 = 0b0000_1000;
const OBJ_SIZE_MASK: u8 = 0b0000_0100;
const OBJ_ENABLE_MASK: u8 = 0b0000_0010;
const BG_WINDOW_ENABLE_MASK: u8 = 0b0000_0001;

#[derive(Default)]
pub struct LcdControl {
    ppu_enable: bool,
    window_tile_map_area: bool,
    window_enable: bool,
    bg_window_tile_data_area: bool,
    bg_tile_map_area: bool,
    obj_size_8x16: bool,
    obj_enable: bool,
    bg_window_enable: bool,
}

impl LcdControl {
    pub fn enable_ppu(&mut self, enable: bool) {
        self.ppu_enable = enable;
    }

    pub fn select_window_tile_map(&mut self, use_map1: bool) {
        self.window_tile_map_area = use_map1;
    }

    pub fn enable_window(&mut self, enable: bool) {
        self.window_enable = enable;
    }

    pub fn select_bg_window_tile_data(&mut self, use_data1: bool) {
        self.bg_window_tile_data_area = use_data1;
    }

    pub fn select_bg_tile_map(&mut self, use_map1: bool) {
        self.bg_tile_map_area = use_map1;
    }

    pub fn set_obj_size_8x16(&mut self, is_8x16: bool) {
        self.obj_size_8x16 = is_8x16;
    }

    pub fn enable_obj(&mut self, enable: bool) {
        self.obj_enable = enable;
    }

    pub fn enable_bg_window(&mut self, enable: bool) {
        self.bg_window_enable = enable;
    }

    pub fn is_ppu_enabled(&self) -> bool {
        self.ppu_enable
    }

    pub fn window_tile_map_area(&self) -> Range<u16> {
        if self.window_tile_map_area {
            TILE_MAP_1
        } else {
            TILE_MAP_0
        }
    }

    pub fn is_window_enabled(&self) -> bool {
        self.window_enable
    }

    pub fn bg_window_tile_data_area(&self) -> Range<u16> {
        if self.bg_window_tile_data_area {
            TILE_DATA_1
        } else {
            TILE_DATA_0
        }
    }

    pub fn bg_tile_map_area(&self) -> Range<u16> {
        if self.bg_tile_map_area {
            TILE_MAP_1
        } else {
            TILE_MAP_0
        }
    }

    pub fn is_obj_size_8x16(&self) -> bool {
        self.obj_size_8x16
    }

    pub fn is_obj_enabled(&self) -> bool {
        self.obj_enable
    }

    pub fn is_bg_window_enabled(&self) -> bool {
        self.bg_window_enable
    }

    pub fn update_from_byte(&mut self, value: u8) {
        self.ppu_enable = value & PPU_ENABLE_MASK != 0;
        self.window_tile_map_area = value & WINDOW_TILE_MAP_MASK != 0;
        self.window_enable = value & WINDOW_ENABLE_MASK != 0;
        self.bg_window_tile_data_area = value & BG_WINDOW_TILE_DATA_MASK != 0;
        self.bg_tile_map_area = value & BG_TILE_MAP_MASK != 0;
        self.obj_size_8x16 = value & OBJ_SIZE_MASK != 0;
        self.obj_enable = value & OBJ_ENABLE_MASK != 0;
        self.bg_window_enable = value & BG_WINDOW_ENABLE_MASK != 0;
    }
}
