#![allow(unused_variables)]
#![allow(dead_code)]

pub mod lcd_control;
pub mod lcd_status;

use crate::mmu::Mmu;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::lcd_status::LcdStatus;
use std::cell::RefCell;
use std::rc::Rc;

const VRAM_START: u16 = 0x8000; // Start of VRAM

#[derive(Default)]
pub struct Ppu {
    pub bus: Rc<RefCell<Mmu>>,
    lcd_control: LcdControl,
    lcd_status: LcdStatus, // LCD Status register
    scx: u8,               // Scroll X
    scy: u8,               // Scroll Y
    wy: u8,                // Window Y position
    wx: u8,                // Window X position
    ly: u8,
    lyc: u8,
}

impl Ppu {
    pub fn new(bus: Rc<RefCell<Mmu>>) -> Self {
        Ppu {
            bus,
            lcd_control: LcdControl::default(),
            lcd_status: LcdStatus::new(),
            scx: 0x00,
            scy: 0x00,
            wy: 0x00,
            wx: 0x00,
            ly: 0x00,
            lyc: 0x00,
        }
    }

    pub fn display_vram(&self) {
        for i in 0..0x2000 {
            let byte = self.bus.borrow().read_byte(VRAM_START + i as u16);
            print!("{:02X} ", byte);
            if (i + 1) % 16 == 0 {
                println!();
            }
        }
    }

    pub fn display_tiles_data(&self) {
        println!("Tile Data Area:");
        for tile_index in 0..384 {
            // 384 tiles, each 16 bytes
            let tile_address = VRAM_START + (tile_index as u16 * 16);
            print!("{:04x} Tile {:03}: ", tile_address, tile_index);
            for byte_index in 0..16 {
                let byte = self
                    .bus
                    .borrow()
                    .read_byte(tile_address + byte_index as u16);
                print!("{:02X} ", byte);
            }
            println!();
        }
    }

    pub fn display_tile_map_area(&self, tile_map_address: u16) {
        println!("Tile Map Area at 0x{:04X}:", tile_map_address);
        for y in 0..32 {
            for x in 0..32 {
                let offset = (y * 32 + x) as u16;
                let tile_number = self.bus.borrow().read_byte(tile_map_address + offset);
                print!("{:02X} ", tile_number);
            }
            println!();
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
            2 => [96, 96, 96],    // Dark Gray
            3 => [0, 0, 0],       // Black
            _ => [0, 0, 0],
        }
    }

    pub fn read_tile_data(&self, tile_address: u16) -> [u8; 16] {
        let mut tile_data = [0; 16];

        for (i, byte) in tile_data.iter_mut().enumerate() {
            *byte = self.bus.borrow().read_byte(tile_address + i as u16);
        }

        tile_data
    }

    pub fn display_all_tiles(&self) -> Vec<u8> {
        let mut frame = vec![0; 160 * 144 * 3];
        for y in 0..144 {
            for x in 0..160 {
                let tile_index = (y / 8) * 20 + (x / 8);
                let base_address = VRAM_START + (tile_index as u16 * 16);
                let tile_data = self.read_tile_data(base_address);
                let color = self.get_pixel_color(tile_data, x, y);
                let offset = (y * 160 + x) * 3;
                frame[offset] = color[0];
                frame[offset + 1] = color[1];
                frame[offset + 2] = color[2];
            }
        }
        frame
    }

    pub fn render_frame(&self) -> Vec<u8> {
        let mut frame = vec![0; 160 * 144 * 3];
        for y in 0..144 {
            for x in 0..160 {
                let y_tile = y / 8;
                let x_tile = x / 8;
                let tilemap_base: std::ops::Range<u16> = self.lcd_control.get_bg_tile_map();

                let offset = (y_tile * 32 + x_tile) as u16;
                let tile_number = self.bus.borrow().read_byte(tilemap_base.start + offset);
                let tile_address = if self.lcd_control.get_bg_window_tiles() {
                    0x8000 + (tile_number as u16) * 16
                } else {
                    0x8080 + ((tile_number as i8) as u16 + 128) * 16
                };

                let tile_data = self.read_tile_data(tile_address);
                let color = self.get_pixel_color(tile_data, x, y);
                let color_offset = (y * 160 + x) * 3;
                frame[color_offset] = color[0];
                frame[color_offset + 1] = color[1];
                frame[color_offset + 2] = color[2];
            }
        }
        frame
    }

    pub fn update_registers(&mut self) {
        self.ly = self.bus.borrow().read_byte(0xFF44);
        self.lyc = self.bus.borrow().read_byte(0xFF45);
        self.scy = self.bus.borrow().read_byte(0xFF42);
        self.scx = self.bus.borrow().read_byte(0xFF43);
        self.wy = self.bus.borrow().read_byte(0xFF4A);
        self.wx = self.bus.borrow().read_byte(0xFF4B);
        self.lcd_control.update(self.bus.borrow().read_byte(0xFF40));
    }
}
