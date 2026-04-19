#![allow(unused_variables)]
#![allow(dead_code)]

mod colors_palette;
mod lcd_control;
mod lcd_status;
mod pixel;
mod pixel_fifo;
mod obj_piso;
mod pixel_fetcher;
mod oam_fetcher;

use std::sync::Mutex;
use std::sync::{Arc, RwLock};

use crate::mmu::mbc::Mbc;
use crate::mmu::MemoryRegion;
use crate::mmu::Mmu;
use crate::mmu::oam::Sprite;
use crate::mmu::interrupt::Interrupt;
use crate::ppu::colors_palette::Color;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::lcd_status::LcdStatus;
use crate::ppu::lcd_status::PpuMode;
use crate::ppu::pixel::Pixel;
use crate::ppu::pixel_fifo::PixelFifo;
use crate::ppu::obj_piso::ObjPiso;
use crate::ppu::pixel_fetcher::PixelFetcher;
use crate::ppu::oam_fetcher::OamFetcher;

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
const BGP_ADDR: u16 = 0xFF47; // Background Palette
const OBP0_ADDR: u16 = 0xFF48; // Object Palette 0
const OBP1_ADDR: u16 = 0xFF49; // Object Palette 1
const WY_ADDR: u16 = 0xFF4A; // Window Y Position
const WX_ADDR: u16 = 0xFF4B; // Window X Position
const LCD_CONTROL_ADDR: u16 = 0xFF40; // LCDC Control

const OAM_DOTS: u32 = 80; // always 80
const PIXEL_TRANSFER_DOTS: u32 = 172; // can change between 172 and 289, to handle later
const HBLANK_DOTS: u32 = 204; // can change between 87 and 204, to handle later
const SCANLINE_DOTS: u32 = 456; // always 456

pub struct Ppu<T: Mbc> {
    pub bus: Arc<RwLock<Mmu<T>>>,
    pub dots: u32,
    lcd_control: LcdControl,
    lcd_status: LcdStatus, // LCD Status register
    scx: u8,               // Scroll X
    scy: u8,               // Scroll Y
    wy: u8,                // Window Y position
    wx: u8,                // Window X position
    wly: u8,               // Window internal line counter
    ly: u8,
    lyc: u8,
    x: usize,
    fetcher: PixelFetcher,
    bg_fifo: PixelFifo, // Background pixel FIFO
    visible_sprites: [Option<Sprite>; 10],
    pixels_to_discard: u8, // Required in order to prevent the SCX misalignment bug
    use_window: bool, // Required for BG FIFO in order to know if the window is activated midline
    wx_at_window_start: u8, // Required to handle the WX hardware glitch
    is_wx_glitch_happened: bool, // Required to handle the WX hardware glitch
    bg_color_indices: [u8; 160], // tmp until FIFO OBJ
}

impl<T: Mbc> Ppu<T> {
    pub fn new(bus: Arc<RwLock<Mmu<T>>>) -> Self {
        Ppu {
            bus,
            dots: 0,
            lcd_control: LcdControl::default(),
            lcd_status: LcdStatus::new(),
            scx: 0x00,
            scy: 0x00,
            wy: 0x00,
            wx: 0x00,
            wly: 0x00,
            ly: 0x00,
            lyc: 0x00,
            x: 0,
            fetcher: PixelFetcher::default(),
            bg_fifo: PixelFifo::default(),
            visible_sprites: [None; 10],
            pixels_to_discard: 0,
            use_window: false,
            wx_at_window_start: 0x00,
            is_wx_glitch_happened: false,
            bg_color_indices: [0u8; 160],
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


    pub fn get_pixel_color_index(&self, tile_data: [u8; 16], x: usize, y: usize) -> u8 {
        /*
            Every tile is 8x8 pixels, 2 bits/pixels
            Every line is 2 bytes:
                - low weight bit (LSB)
                - heavy weight bit (MSB)
            The final pixel is (MSB << 1) | LSB
        */

        let pixel_x = x % 8;
        let pixel_y = y % 8;

        let lsb_byte = tile_data[pixel_y * 2];
        let msb_byte = tile_data[pixel_y * 2 + 1];

        let bit_index = 7 - pixel_x;

        let lsb_bit = (lsb_byte >> bit_index) & 1;
        let msb_bit = (msb_byte >> bit_index) & 1;

        (msb_bit << 1) | lsb_bit
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
                let color = Color::from_index(self.get_pixel_color_index(tile_data, x, y));
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
        /*
            The Game Boy has 2 tilemaps (BG and Windows)
            and 2 addressing modes for tiles:
                - 0x8000 (unsigned index)
                - 0x8800/0x9000 (signed index)
        */

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
            // Unsigned mode: simple multiplication
            lcd_control::TILE_DATA_1 => 0x8000 + (tile_number as u16) * 16,
            // Signed mode: tile_number is interpreted as i8 ([-128;127]), base = 0x9000
            lcd_control::TILE_DATA_0 => {
                let base = 0x9000u16;
                let offset = (tile_number as i8) as i16 * 16;
                base.wrapping_add_signed(offset)
            },
            _ => unreachable!(),
        }
    }

    fn oam_search(&mut self) {
        // Select max 10 visible sprites on the scanline

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

    fn apply_background_palette(&self, color_index: u8) -> Color {
        let palette = self.bus.read().unwrap().read_byte(BGP_ADDR);

        let index = (palette >> (color_index * 2)) & 0b11;

        Color::from_index(index)
    }

    fn render_background(&self) -> Vec<Pixel> {
        /*
            Generate one complete scanline (160 pixels) by applying:
                - scroll (SCX/SCY)
                - window (WX/WY)
                - wrapping (256x256)
         */

        let mut pixels = Vec::new();
        let ly = self.ly as usize; // line to render
        let default_color = self.apply_background_palette(0);

        for x in 0..WIN_SIZE_X {
            // If BG is disabled, color 0 everywhere
            if !self.lcd_control.is_bg_window_enabled() {
                pixels.push(Pixel::new_bg(default_color, 0));
                continue;
            }

            // Hardware condition to enable the window
            // WX is shifted by 7 pixels
            let use_window = self.lcd_control.is_window_enabled()
                && (ly >= self.wy as usize)
                && (x + 7 >= self.wx as usize);

            let (bg_x, bg_y) = if use_window {
                let win_x = x + 7 - self.wx as usize;
                let win_y = self.wly as usize;
                (win_x % 256, win_y % 256)
            } else {
                // coordinates with scrool (wrap 256)
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

            let color_index = self.get_pixel_color_index(tile, pixel_x, pixel_y);
            let color = self.apply_background_palette(color_index);
            let pixel = Pixel::new_bg(color, color_index);
            
            pixels.push(pixel);
        }

        pixels
    }

    fn extract_attributes(&self, attributes: u8) -> (bool, bool, bool, bool) {
        (
            ((attributes >> 7) & 1) != 0,
            ((attributes >> 6) & 1) != 0,
            ((attributes >> 5) & 1) != 0,
            ((attributes >> 4) & 1) != 0,
        )
    }

    fn get_sprite_tile(&self, height: u8, sprite: Sprite, actual_sprite_line: usize) -> [u8; 16] {
        let tile_always_pair = if height == 16 { sprite.tile & 0xFE } else { sprite.tile };
        let tile_index = if height == 16 && actual_sprite_line >= 8 { tile_always_pair + 1 } else { tile_always_pair }; // offset if 8x16 because of end of tile index
        let tile_address = VRAM.to_address() + (tile_index as u16 * 16);
        
        self.read_tile_data(tile_address)
    }

    fn get_right_pixel(&self, color_index: u8, color: Color, priority: bool) -> Option<Color> {
        // Deal with sprite/background priority
        //TODO tmp function to handle the transition between scanline rendering and FIFO.
        // right now only fifo background is implemented. In order to keep render_sprites working
        // we need to keep and adapt this function.

        // if old_pixel.get_is_sprite() {
        //     return None
        // }

        if priority && color_index != 0 {
            return None
        }

        // Some(Pixel::new(color, true, color_index))
        Some(color)
    }

    fn apply_sprite_palette(&self, color_index: u8, palette_attribute: bool) -> Color {
        let palette_addr = if palette_attribute { OBP1_ADDR } else { OBP0_ADDR };
        let palette = self.bus.read().unwrap().read_byte(palette_addr);

        let index = (palette >> (color_index * 2)) & 0b11;

        Color::from_index(index)
    }

    fn sort_sprites_by_x(&self) -> Vec<Sprite> {
        let mut sprites: Vec<(usize, Sprite)> = self.visible_sprites
            .iter()
            .enumerate()
            .filter_map(| (i, s) | s.map(| sprite | (i, sprite)))
            .collect();

        sprites.sort_by(| (index_a, sprite_a), (index_b, sprite_b) | {
            if sprite_a.x != sprite_b.x {
                sprite_a.x.cmp(&sprite_b.x)
            } else {
                index_a.cmp(index_b)
            }
        });

        sprites.into_iter().map(| (_, s) | s).collect()
    }

    fn render_sprites(&self, image: &mut Arc<Mutex<Vec<u8>>>) {
        /*
            TODO Transition modification until the FIFO OBJ is implemented. Right now the modifications
            should make the function works while we test the FIFO background

            Apply sprites above background
            respect:
                - priority
                - flip X/Y
                - palettes
                - transparency
        */

        let height: u8 = if self.lcd_control.is_obj_size_8x16() { 16 } else { 8 };

        // sort by X then OAM order (hardware behavior)
        let sorted_sprites = self.sort_sprites_by_x();

        for sprite in sorted_sprites {
            if !self.lcd_control.is_obj_enabled() {
                continue;
            }

            let (priority, y_flip, x_flip, palette_attribute) = self.extract_attributes(sprite.attributes);

            // sprite coordinates are shifted: Y - 16, X - 8
            let sprite_top: i16 = sprite.y as i16 - 16;
            let sprite_line = (self.ly as i16 - sprite_top) as usize;

            let actual_sprite_line = if y_flip { (height as usize - 1) - sprite_line } else { sprite_line };
            let tile = self.get_sprite_tile(height, sprite, actual_sprite_line);

            for pixel_x in 0..8 {
                let screen_x = (sprite.x - 8 + pixel_x) as i16;
                    
                if !(0..160).contains(&screen_x) {
                    continue;
                }

                let actual_pixel_x = if x_flip { 7 - pixel_x } else { pixel_x };
                let color_index = self.get_pixel_color_index(tile, actual_pixel_x as usize, actual_sprite_line % 8); // % 8 to handle 8x16

                // 0 = transparency for sprites 
                if color_index == 0 { continue; }
                    
                let color = self.apply_sprite_palette(color_index, palette_attribute);

                if let Some(new_pixel) = self.get_right_pixel(self.bg_color_indices[screen_x as usize], color, priority) {
                    let offset = (self.ly as usize * WIN_SIZE_X + screen_x as usize) * 3;
                    let mut frame = image.lock().unwrap();

                    self.set_pixel_color(&mut frame, offset, new_pixel);
                }
            }
        }
    }


    fn mode_oam_search(&mut self) -> bool {
        if self.dots >= OAM_DOTS {
            self.visible_sprites = [None; 10];
            self.oam_search();

            self.lcd_status.update_ppu_mode(PpuMode::PixelTransfer);
        }
        false
    }

    fn mode_pixel_transfer(&mut self, image: &mut Arc<Mutex<Vec<u8>>>) -> bool {
        if self.ly < WIN_SIZE_Y as u8 {
            // let mut pixels = self.render_background();

            let use_window = self.lcd_control.is_window_enabled()
                && (self.ly as usize >= self.wy as usize)
                && (self.x + 7 >= self.wx as usize);    

            // check if window is activated in the middle of scanline
            if !self.use_window && use_window {
                self.fetcher.reset();
                self.bg_fifo.clear();

                self.use_window = use_window;
                self.wx_at_window_start = self.wx;

                self.pixels_to_discard = 0;
            }

            if self.use_window && self.wx != self.wx_at_window_start
                && self.x + 7 >= self.wx as usize
                && !self.is_wx_glitch_happened {
                    let glitched_pixel = Pixel::new_bg(self.apply_background_palette(0),  0);

                    self.bg_fifo.push(glitched_pixel);
                    self.is_wx_glitch_happened = true;
            }

            let tile_pixels = self.fetcher.tick(&self.bus, &self.bg_fifo, self.ly, self.scx, self.scy, &self.lcd_control, use_window);
            
            if let Some(pixels) = tile_pixels {
                for pixel in pixels {
                    self.bg_fifo.push(pixel);
                }
            }

            {
                if let Some(current_pixel) = self.bg_fifo.pop() {
                    if self.pixels_to_discard > 0 {
                        self.pixels_to_discard -= 1;
                    } else {
                        let color_index: u8;
                        let color: Color;

                        // If BG is disabled, color 0 everywhere
                        if !self.lcd_control.is_bg_window_enabled() {
                            color_index = 0;
                            color = self.apply_background_palette(0);
                        }
                        else {
                            color_index = current_pixel.get_color_index();
                            color = *current_pixel.get_color();
                        }
                        self.bg_color_indices[self.x] = color_index;

                        let mut frame = image.lock().unwrap();
                        let ly = self.ly as usize;

                        let offset = (ly * WIN_SIZE_X + self.x) * 3; // * 3 for each pixels (3 bytes (RGB))
                        self.set_pixel_color(&mut frame, offset, color);

                        self.x += 1;
                    }
                }

            }
        }

        if self.x == 160 {
            self.render_sprites(image);
            self.lcd_status.update_ppu_mode(PpuMode::HBlank);
        }

        false
    }

    fn mode_hblank(&mut self) -> bool {
        // End of scanline -> next one after 456 dots

        if self.dots >= SCANLINE_DOTS {
            self.dots -= SCANLINE_DOTS;

            if self.lcd_control.is_window_enabled()
                && self.ly >= self.wy
                && self.wx <= 166 {

                self.wly += 1;
            }
            
            self.ly += 1;
            self.check_lyc_equals_ly();

            // reset for newline
            // TODO proper reset function
            self.bg_color_indices = [0; 160];
            self.x = 0;
            self.bg_fifo.clear();
            self.fetcher.reset();
            self.pixels_to_discard = self.scx % 8;
            self.use_window = false;
            self.is_wx_glitch_happened = false;

            if self.ly >= WIN_SIZE_Y as u8 {
                self.lcd_status.update_ppu_mode(PpuMode::VBlank);

                self.bus.write().unwrap().interrupts_request(Interrupt::VBlank);

                return true;
            } else {
                self.lcd_status.update_ppu_mode(PpuMode::OamSearch);
            }
        }
        false
    }

    fn mode_vblank(&mut self) -> bool {
        if self.dots >= SCANLINE_DOTS {
            self.dots -= SCANLINE_DOTS;
            self.ly += 1;
            self.check_lyc_equals_ly();

            if self.ly >= WIN_SIZE_Y as u8 + VBLANK_SIZE as u8 {
                self.ly = 0;
                self.check_lyc_equals_ly();

                self.wly = 0;

                // TODO proper reset function
                self.bg_color_indices = [0; 160];
                self.x = 0;
                self.bg_fifo.clear();
                self.fetcher.reset();
                self.pixels_to_discard = self.scx % 8;
                self.use_window = false;
                self.is_wx_glitch_happened = false;

                self.lcd_status.update_ppu_mode(PpuMode::OamSearch);
            }
        }
        false
    }

    pub fn tick(&mut self, cycles: u32,  image: &mut Arc<Mutex<Vec<u8>>>) -> bool {
        self.update_registers();
        self.dots += cycles;

        match self.lcd_status.get_ppu_mode() {
            PpuMode::OamSearch => self.mode_oam_search(),
            PpuMode::PixelTransfer => self.mode_pixel_transfer(image),
            PpuMode::HBlank => self.mode_hblank(),
            PpuMode::VBlank => self.mode_vblank(),
        }
    }

    fn check_lyc_equals_ly(&mut self) {
        /*
            LYC == LY is an hardware condition:
                - update a flag in STAT
                - can trigger a LCD STAT interrupt
            It's used by many games to synchronize with scanline
        */
        let lyc_match = self.ly == self.lyc;
        self.lcd_status.set_lyc_equals_ly(lyc_match);
        
        if lyc_match && self.lcd_status.get_lyc_equals_ly() {
            self.bus.write().unwrap().interrupts_request(Interrupt::LcdStat);
        }
    }

    pub fn update_registers(&mut self) {
        self.lyc = self.bus.read().unwrap().read_byte(LYC_ADDR);
        self.scy = self.bus.read().unwrap().read_byte(SCY_ADDR);
        self.scx = self.bus.read().unwrap().read_byte(SCX_ADDR);
        self.wy = self.bus.read().unwrap().read_byte(WY_ADDR);
        self.wx = self.bus.read().unwrap().read_byte(WX_ADDR);

        // LCDC control the whole PPU's behavior
        self.lcd_control
            .update_from_byte(self.bus.read().unwrap().read_byte(LCD_CONTROL_ADDR));
        
        // STAT is an hybrid register, some bits are controlled by PPU and other by CPU

        // Write internal state in memory
        let stat_byte = self.lcd_status.struct_to_byte();
        self.bus.write().unwrap().write_byte(STAT_ADDR, stat_byte);

        // Read state from memory to get the modifications made by CPU
        let stat_from_mmu = self.bus.read().unwrap().read_byte(STAT_ADDR);
        self.lcd_status.update_from_byte(stat_from_mmu);

        // Current line
        self.bus.write().unwrap().write_byte(LY_ADDR, self.ly);
    }
}
