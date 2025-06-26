#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Range;

#[derive(Default)]
pub struct LcdControl {
    ppu_enable: bool,
    window_tile_map_area: bool,
    window_enable: bool,
    bg_window_tiles: bool,
    bg_tile_map: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_window_enable: bool,
}

impl LcdControl {
    pub fn set_ppu_enable(&mut self, enable: bool) {
        self.ppu_enable = enable;
    }

    pub fn set_window_tile_map_area(&mut self, area: bool) {
        self.window_tile_map_area = area;
    }

    pub fn set_window_enable(&mut self, enable: bool) {
        self.window_enable = enable;
    }

    pub fn set_bg_window_tiles(&mut self, tiles: bool) {
        self.bg_window_tiles = tiles;
    }

    pub fn set_bg_tile_map(&mut self, map: bool) {
        self.bg_tile_map = map;
    }

    pub fn set_obj_size(&mut self, size: bool) {
        self.obj_size = size;
    }

    pub fn set_obj_enable(&mut self, enable: bool) {
        self.obj_enable = enable;
    }

    pub fn set_bg_window_enable(&mut self, enable: bool) {
        self.bg_window_enable = enable;
    }

    pub fn get_ppu_enable(&self) -> bool {
        self.ppu_enable
    }

    pub fn get_window_tile_map_area(&self) -> Range<u16> {
        if self.window_tile_map_area {
            0x9C00..0x9F00
        } else {
            0x9800..0x9BFF
        }
    }

    pub fn get_window_enable(&self) -> bool {
        self.window_enable
    }

	pub fn get_bg_window_tiles(&self) -> bool {
		self.bg_window_tiles
	}

    pub fn get_bg_window_tiles_area(&self) -> Range<u16> {
        if self.bg_window_tiles {
            0x8000..0x8FFF
        } else {
            0x8800..0x97FF
        }
    }

    pub fn get_bg_tile_map(&self) -> Range<u16> {
        if self.bg_tile_map {
            0x9c00..0x9FFF
        } else {
            0x9800..0x9BFF
        }
    }

    pub fn get_obj_size(&self) -> bool {
        self.obj_size
    }

    pub fn get_obj_enable(&self) -> bool {
        self.obj_enable
    }
    pub fn get_bg_window_enable(&self) -> bool {
        self.bg_window_enable
    }

    pub fn update(&mut self, value: u8) {
        self.ppu_enable = value & 0b1000_0000 != 0;
        self.window_tile_map_area = value & 0b0100_0000 != 0;
        self.window_enable = value & 0b0010_0000 != 0;
        self.bg_window_tiles = value & 0b0001_0000 != 0;
        self.bg_tile_map = value & 0b0000_1000 != 0;
        self.obj_size = value & 0b0000_0100 != 0;
        self.obj_enable = value & 0b0000_0010 != 0;
        self.bg_window_enable = value & 0b0000_0001 != 0;
    }
}
