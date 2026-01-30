#![allow(unused_variables)]
#![allow(dead_code)]

mod colors_palette;
mod lcd_control;
mod lcd_status;
mod pixel;
mod pixel_fifo;

use std::sync::Mutex;

use crate::mmu::MemoryRegion;
use crate::mmu::Mmu;
use crate::mmu::oam::Sprite;
use crate::ppu::colors_palette::Color;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::lcd_status::LcdStatus;
use crate::ppu::lcd_status::PpuMode;
use crate::ppu::pixel::Pixel;
use crate::ppu::pixel_fifo::PixelFifo;
use std::sync::{Arc, RwLock};

pub const WIN_SIZE_X: usize = 160; // Window size in X direction
pub const WIN_SIZE_Y: usize = 144; // Window size in Y direction
pub const VBLANK_SIZE: usize = 10; // VBlank size in lines
const VRAM: MemoryRegion = MemoryRegion::Vram; // Start of VRAM
const OAM: MemoryRegion = MemoryRegion::Oam; // Start of OAM
const LY_ADDR: u16 = 0xFF44; // LCDC Y-Coordinate
const LYC_ADDR: u16 = 0xFF45; // LY Compare
const STAT_ADDR: u16 = 0xFF41; // LCDC Status
const SCX_ADDR: u16 = 0xFF43; // Scroll X
const SCY_ADDR: u16 = 0xFF42; // Scroll Y
const WY_ADDR: u16 = 0xFF4A; // Window Y Position
const WX_ADDR: u16 = 0xFF4B; // Window X Position
const LCD_CONTROL_ADDR: u16 = 0xFF40; // LCDC Control

#[derive(Default)]
pub struct Ppu {
    pub bus: Arc<RwLock<Mmu>>,
    lcd_control: LcdControl,
    lcd_status: LcdStatus, // LCD Status register
    scx: u8,               // Scroll X
    scy: u8,               // Scroll Y
    wy: u8,                // Window Y position
    wx: u8,                // Window X position
    ly: u8,
    lyc: u8,
    x: usize,
    bg_fifo: PixelFifo, // Background pixel FIFO
    visible_sprites: [Option<Sprite>; 10],
}

impl Ppu {
    pub fn new(bus: Arc<RwLock<Mmu>>) -> Self {
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
            x: 0,
            bg_fifo: PixelFifo::default(),
            visible_sprites: [None; 10],
        }
    }

    pub fn display_vram(&self) {
        for i in 0..0x2000 {
            let byte = self
                .bus
                .read()
                .unwrap()
                .read_byte(VRAM.to_address() + i as u16);
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
                    .read()
                    .unwrap()
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
                let tile_number = self
                    .bus
                    .read()
                    .unwrap()
                    .read_byte(tile_map_address + offset);
                print!("{tile_number:02X} ");
            }
            println!();
        }
    }

    pub fn get_pixel_color(&self, tile_data: [u8; 16], x: usize, y: usize) -> Color {
        let pixel_x = x % 8;
        let pixel_y = y % 8;

        let lsb_byte = tile_data[pixel_y * 2];
        let msb_byte = tile_data[pixel_y * 2 + 1];

        let bit_index = 7 - pixel_x;

        let lsb_bit = (lsb_byte >> bit_index) & 1;
        let msb_bit = (msb_byte >> bit_index) & 1;

        let color_index = (msb_bit << 1) | lsb_bit;

        Color::from_index(color_index)
    }

    pub fn read_tile_data(&self, tile_address: u16) -> [u8; 16] {
        let mut tile_data = [0; 16];

        for (i, byte) in tile_data.iter_mut().enumerate() {
            *byte = self.bus.read().unwrap().read_byte(tile_address + i as u16);
        }

        tile_data
    }

    pub fn render_all_tiles(&self) -> Vec<u8> {
        let mut frame = vec![0; WIN_SIZE_X * WIN_SIZE_Y * 3];
        for y in 0..WIN_SIZE_Y {
            for x in 0..WIN_SIZE_X {
                let tile_index = (y / 8) * 20 + (x / 8);
                let base_address = VRAM.to_address() + (tile_index as u16 * 16);
                let tile_data = self.read_tile_data(base_address);
                let color = self.get_pixel_color(tile_data, x, y);
                let offset = (y * 160 + x) * 3;
                self.set_pixel_color(&mut frame, offset, color);
            }
        }
        frame
    }

    fn set_pixel_color(&self, frame: &mut [u8], offset: usize, color: Color) {
        let color_rgb = color.to_rgb();
        frame[offset] = color_rgb[0];
        frame[offset + 1] = color_rgb[1];
        frame[offset + 2] = color_rgb[2];
    }

    fn get_tile_address(&self, y: usize, x: usize, use_window: bool) -> u16 {
        let tilemap_base: std::ops::Range<u16> = if use_window {
            self.lcd_control.window_tile_map_area()
        } else {
            self.lcd_control.bg_tile_map_area()
        };

        let offset = (y * 32 + x) as u16;
        let tile_number = self
            .bus
            .read()
            .unwrap()
            .read_byte(tilemap_base.start + offset);
        match self.lcd_control.bg_window_tile_data_area() {
            lcd_control::TILE_DATA_1 => 0x8000 + (tile_number as u16) * 16,
            lcd_control::TILE_DATA_0 => 0x8800 + ((tile_number as i8 as i16) as u16) * 16,
            _ => unreachable!(),
        }
    }

    fn oam_search(&mut self) {
        let height:u8 = if self.lcd_control.is_obj_size_8x16() {
            16
        } else {
            8
        };
        let mmu = self.bus.read().unwrap();
        let oam  = mmu.get_oam();
        let mut i: usize = 0;
        for sprite in &oam.sprites {
            if sprite.is_visible(self.ly, height) {
                self.visible_sprites[i] = Some(*sprite);
                i += 1;
                if i >= 10 {
                    break;
                }
            }
        }
    }

   fn render_background(&self) -> Vec<Pixel> {
        let mut pixels = Vec::new();
        let ly = self.ly as usize; // line to render

        for x in 0..WIN_SIZE_X {
            let use_window = self.lcd_control.is_window_enabled()
                && (ly >= self.wy as usize)
                && (x + 7 >= self.wx as usize);

            let (bg_x, bg_y) = if use_window {
                let win_x = x + 7 - self.wx as usize;
                let win_y = ly - self.wy as usize;
                (win_x % 256, win_y % 256)
            } else {
                (
                    (x + self.scx as usize) % 256,
                    (ly + self.scy as usize) % 256,
                )
            };

            let tile_x = bg_x / 8;
            let tile_y = bg_y / 8;
            let tile_address = self.get_tile_address(tile_y, tile_x, use_window);
            let tile = self.read_tile_data(tile_address);
            let pixel_x = bg_x % 8;
            let pixel_y = bg_y % 8;
            let color = self.get_pixel_color(tile, pixel_x, pixel_y);
            let pixel = Pixel::new(color, 0, 0, 0);
            
            pixels.push(pixel);
        }

        pixels
    }

    fn extract_attributes(attributes: u8) -> (bool, bool, bool, bool) {
        (
            ((1 << 0) & attributes) != 0,
            ((1 << 1) & attributes) != 0,
            ((1 << 2) & attributes) != 0,
            ((1 << 3) & attributes) != 0,
        )
    }

    fn render_sprites(&self, mut pixels: Vec<Pixel>) -> Vec<Pixel> {
        // Pour chaque sprite visible
        // Pour chaque pixel X du sprite (0-7)
        // Calculer la position dans le Vec (0-159)
        // Si pixel sprite non transparent
        // Remplacer pixels[position]
        for sprite_option in self.visible_sprites {
            if let Some(sprite) = sprite_option {
                let color = self.get_pixel_color(sprite.tile, sprite.x, sprite.y);
                let pixel = Pixel::new(color, 0, 0, 0);

            }
        }
        pixels
    }

    pub fn render_frame(&mut self, image: &mut Arc<Mutex<Vec<u8>>>) -> bool {
        if self.ly < WIN_SIZE_Y as u8 {
            self.lcd_status.update_ppu_mode(PpuMode::OamSearch);
            self.visible_sprites = [None; 10];
            self.oam_search();
            
            let mut pixels = self.render_background();
            pixels = self.render_sprites(pixels);

            {
                let mut frame = image.lock().unwrap();
                let ly = self.ly as usize;

                for (x, p) in pixels.into_iter().enumerate() {
                    let offset = (ly * WIN_SIZE_X + x) * 3; // * 3 for each pixels (3 bytes (RGB))
                    self.set_pixel_color(&mut frame, offset, *p.get_color());
                }
            }
        }
        
        self.ly += 1;

        if self.ly >= WIN_SIZE_Y as u8 + VBLANK_SIZE as u8 {
            // Reset
            self.ly = 0;
        }
        if self.ly >= WIN_SIZE_Y as u8 {
            // Lines 144-153: VBlank
            self.lcd_status.update_ppu_mode(PpuMode::VBlank);
            false
        } else {
            // Lines 0-143: end line: HBlank
            self.lcd_status.update_ppu_mode(PpuMode::HBlank);
            true
        }    
    }

    pub fn update_registers(&mut self) {
        self.lyc = self.bus.read().unwrap().read_byte(LYC_ADDR);
        self.scy = self.bus.read().unwrap().read_byte(SCY_ADDR);
        self.scx = self.bus.read().unwrap().read_byte(SCX_ADDR);
        self.wy = self.bus.read().unwrap().read_byte(WY_ADDR);
        self.wx = self.bus.read().unwrap().read_byte(WX_ADDR);
        self.lcd_control
            .update_from_byte(self.bus.read().unwrap().read_byte(LCD_CONTROL_ADDR));
    }
}
