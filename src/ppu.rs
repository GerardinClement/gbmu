#![allow(unused_variables)]
#![allow(dead_code)]

use crate::memory::MemoryBus;
use std::cell::RefCell;
use std::rc::Rc;

const VRAM_START: u16 = 0x8000; // Start of VRAM

#[derive(Default)]
pub struct Ppu {
    pub bus: Rc<RefCell<MemoryBus>>,
    pub lcd_control: u8, // LCD Control register
    pub lcd_status: u8,  // LCD Status register
    pub scx: u8,         // Scroll X
    pub scy: u8,         // Scroll Y
    pub wy: u8,          // Window Y position
    pub wx: u8,          // Window X position
}

impl Ppu {
    pub fn new(bus: Rc<RefCell<MemoryBus>>) -> Self {
        Ppu {
            bus,
            lcd_control: 0x00, // Default value
            lcd_status: 0x00,  // Default value
            scx: 0x00,         // Default value
            scy: 0x00,         // Default value
            wy: 0x00,          // Default value
            wx: 0x00,          // Default value
        }
    }

	pub fn get_pixel_color(&self, tile_data: [u8; 16], x: usize, y: usize) -> [u8; 3] {
		let pixel_x = x % 8;
		let pixel_y = y % 8;
		let lsb = tile_data[pixel_y * 2];
		let msb = tile_data[pixel_y * 2 + 1];
		let color_index = ((msb >> (7 - pixel_x)) & 1) << 1 | ((lsb >> (7 - pixel_x)) & 1);
		match color_index {
			0 => [255, 255, 255], // White
			1 => [192, 192, 192], // Light Gray
			2 => [96, 96, 96],   // Dark Gray
			3 => [0, 0, 0],      // Black
			_ => [0, 0, 0],
		}

	}
	
    pub fn read_tile_data(&self, tile_index: u8) -> [u8; 16] {
        let mut tile_data = [0; 16];
        // let base_address = VRAM_START + (tile_index as u16 * 16);

        // for i in 0..16 {
        //     tile_data[i] = self.bus.borrow().read_byte(base_address + i as u16);
        // }

		tile_data = [
			0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
			0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56, 0x38, 0x7C,
		];

        tile_data
    }

	pub fn render_frame(&self) -> Vec<u8> {
		let mut frame = vec![0; 160 * 144 * 3];
		for y in 0..144 {
			for x in 0..160 {
				let tile_index = (y / 8) * 20 + (x / 8);
				let tile_data = self.read_tile_data(tile_index as u8);
				let color = self.get_pixel_color(tile_data, x, y);
				let offset = (y * 160 + x) * 3;
				frame[offset] = color[0];
				frame[offset + 1] = color[1];
				frame[offset + 2] = color[2];
			}
		}
		frame
	}
}
