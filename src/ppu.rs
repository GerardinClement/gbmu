#![allow(unused_variables)]
#![allow(dead_code)]

pub mod lcd_control;
pub mod lcd_status;

use crate::mmu::MemoryRegion;
use crate::mmu::Mmu;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::lcd_status::LcdStatus;
use std::cell::RefCell;
use std::rc::Rc;

const VRAM: MemoryRegion = MemoryRegion::Vram; // Start of VRAM
const LY_ADDR: u16 = 0xFF44; // LCDC Y-Coordinate
const LYC_ADDR: u16 = 0xFF45; // LY Compare
const STAT_ADDR: u16 = 0xFF41; // LCDC Status
const SCX_ADDR: u16 = 0xFF43; // Scroll X
const SCY_ADDR: u16 = 0xFF42; // Scroll Y
const WY_ADDR: u16 = 0xFF4A; // Window Y Position
const WX_ADDR: u16 = 0xFF4B; // Window X Position
const LCD_CONTROL_ADDR: u16 = 0xFF40; // LCDC Control
const TILE_DATA_0: std::ops::Range<u16> = 0x8800..0x9800; // Tile Data Area 0
const TILE_DATA_1: std::ops::Range<u16> = 0x8000..0x9000; // Tile Data Area 1

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
            let byte = self.bus.borrow().read_byte(VRAM.to_address() + i as u16);
            print!("{byte:02X} ");
            if (i + 1) % 16 == 0 {
                println!();
            }
        }
    }

    pub fn display_tiles_data(&self) {
        println!("Tile Data Area:");
        for tile_index in 0..384 {
            let tile_address = VRAM.to_address() + (tile_index as u16 * 16);
            print!("{tile_address:04x} Tile {tile_index:03}: ");
            for byte_index in 0..16 {
                let byte = self
                    .bus
                    .borrow()
                    .read_byte(tile_address + byte_index as u16);
                print!("{byte:02X} ");
            }
            println!();
        }
    }

    pub fn display_tile_map_area(&self, tile_map_address: u16) {
        println!("Tile Map Area at 0x{tile_map_address:04X}:");
        for y in 0..32 {
            for x in 0..32 {
                let offset = (y * 32 + x) as u16;
                let tile_number = self.bus.borrow().read_byte(tile_map_address + offset);
                print!("{tile_number:02X} ");
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

    pub fn render_all_tiles(&self) -> Vec<u8> {
        let mut frame = vec![0; 160 * 144 * 3];
        for y in 0..144 {
            for x in 0..160 {
                let tile_index = (y / 8) * 20 + (x / 8);
                let base_address = VRAM.to_address() + (tile_index as u16 * 16);
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
                let tilemap_base: std::ops::Range<u16> = self.lcd_control.bg_tile_map_area();

                let offset = (y_tile * 32 + x_tile) as u16;
                let tile_number = self.bus.borrow().read_byte(tilemap_base.start + offset);
                let tile_address = match self.lcd_control.bg_window_tile_data_area() {
                    TILE_DATA_1 => 0x8000 + (tile_number as u16) * 16,
                    TILE_DATA_0 => 0x8800 + ((tile_number as i8 as i16) as u16) * 16,
                    _ => unreachable!(),
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
        self.ly = self.bus.borrow().read_byte(LY_ADDR);
        self.lyc = self.bus.borrow().read_byte(LYC_ADDR);
        self.scy = self.bus.borrow().read_byte(SCY_ADDR);
        self.scx = self.bus.borrow().read_byte(SCX_ADDR);
        self.wy = self.bus.borrow().read_byte(WY_ADDR);
        self.wx = self.bus.borrow().read_byte(WX_ADDR);
        self.lcd_control
            .update_from_byte(self.bus.borrow().read_byte(LCD_CONTROL_ADDR));
    }
}
