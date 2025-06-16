#![allow(unused_variables)]
#![allow(dead_code)]

use crate::memory::MemoryBus;
use std::cell::RefCell;
use std::rc::Rc;

const VRAM_START: u16 = 0x8000; // Start of VRAM

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

	pub fn display_tile_data(&self, tile_index: u8) {
		let tile_data = self.read_tile_data(tile_index);
	
		for row in 0..8 {
			let lsb = tile_data[row * 2];
			let msb = tile_data[row * 2 + 1];
	
			for bit in (0..8).rev() {
				let lsb_bit = (lsb >> bit) & 1;
				let msb_bit = (msb >> bit) & 1;
				let color = (msb_bit << 1) | lsb_bit;
				print!("{}", color);
			}
			println!();
		}
	}
	
    pub fn read_tile_data(&self, tile_index: u8) -> [u8; 16] {
        let mut tile_data = [0; 16];
        let base_address = VRAM_START + (tile_index as u16 * 16);

        for i in 0..16 {
            tile_data[i] = self.bus.borrow().read_byte(base_address + i as u16);
        }

        tile_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_read_tile_data_and_display() {
        let bus = Rc::new(RefCell::new(MemoryBus::default()));
        let mut ppu = Ppu::new(bus.clone());

        // Tile data to test
        let tile_data: [u8; 16] = [
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ];

        // Write tile data to VRAM
        let tile_index = 0x01; // Example tile index
        let base_address = VRAM_START + (tile_index as u16 * 16);
        for (i, &byte) in tile_data.iter().enumerate() {
            bus.borrow_mut().write_byte(base_address + i as u16, byte);
        }

        // Read tile data using PPU
        let read_data = ppu.read_tile_data(tile_index);
        // assert_eq!(read_data, tile_data);

        // Test display_tile_data (output verification is manual)
        println!("Displaying tile data:");
        ppu.display_tile_data(tile_index);
    }
}
