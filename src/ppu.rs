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
    lcd_status: LcdStatus, // LCD Status register
    wly: u8,               // Window internal line counter
    ly: u8,
    x: usize,
    pixel_fetcher: PixelFetcher,
    oam_fetcher: OamFetcher,
    bg_fifo: PixelFifo, // Background pixel FIFO
    obj_piso: ObjPiso, // Objects (sprites) PISO
    visible_sprites: [Option<Sprite>; 10],
    pixels_to_discard: u8, // Required in order to prevent the SCX misalignment bug
    use_window: bool, // Required for BG FIFO in order to know if the window is activated midline
    wx_at_window_start: u8, // Required to handle the WX hardware glitch
    is_wx_glitch_happened: bool, // Required to handle the WX hardware glitch
    fetching_sprite: bool, // pixel fetcher and pixel shifter need to be paused while oam fetcher is called
    current_sprite_to_fetch: Option<usize>,
    wy_equal_ly_condition_met: bool,
}

impl<T: Mbc> Ppu<T> {
    pub fn new(bus: Arc<RwLock<Mmu<T>>>) -> Self {
        Ppu {
            bus,
            dots: 0,
            lcd_status: LcdStatus::new(),
            wly: 0x00,
            ly: 0x00,
            x: 0,
            pixel_fetcher: PixelFetcher::default(),
            oam_fetcher: OamFetcher::default(),
            bg_fifo: PixelFifo::default(),
            obj_piso: ObjPiso::default(),
            visible_sprites: [None; 10],
            pixels_to_discard: 0,
            use_window: false,
            wx_at_window_start: 0x00,
            is_wx_glitch_happened: false,
            fetching_sprite: false,
            current_sprite_to_fetch: None,
            wy_equal_ly_condition_met: false,
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
            self.read_lcdc().window_tile_map_area()
        } else {
            self.read_lcdc().bg_tile_map_area()
        };

        let offset = (y * 32 + x) as u16;
        let tile_number = self
            .bus
            .read()
            .unwrap()
            .read_byte(tilemap_base.start + offset);
        match self.read_lcdc().bg_window_tile_data_area() {
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

        let height:u8 = if self.read_lcdc().is_obj_size_8x16() {
            16
        } else {
            8
        };
        let mmu = self.bus.read().unwrap();
        let oam  = mmu.get_oam();
        let mut i: usize = 0;
        for (oam_index, sprite) in oam.sprites.iter().enumerate() {
            if sprite.is_visible(self.ly, height) {

                let mut s = *sprite;

                s.oam_index = oam_index as u8;


                self.visible_sprites[i] = Some(s);

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


    fn mode_oam_search(&mut self) -> bool {
        if self.dots >= OAM_DOTS {
            self.visible_sprites = [None; 10];
            self.oam_search();

            self.lcd_status.update_ppu_mode(PpuMode::PixelTransfer);
            self.write_stat_to_mmu();
        }

        false
    }

    fn handle_window_switch(&mut self, use_window: bool) {
        // check if window is activated in the middle of scanline
        if !self.use_window && use_window {
            self.pixel_fetcher.reset_for_window();
            self.bg_fifo.clear();

            self.wx_at_window_start = self.read_wx();
            self.pixels_to_discard = 0;
        }

        self.use_window = use_window;

        // check wx glitch
        let wx = self.read_wx();
        if self.use_window && wx != self.wx_at_window_start
            && self.x + 7 >= wx as usize
            && !self.is_wx_glitch_happened {
                let glitched_pixel = Pixel::new_bg(self.apply_background_palette(0),  0);

                self.bg_fifo.push(glitched_pixel);
                self.is_wx_glitch_happened = true;
        }
    }

    fn push_pixel_to_screen(&mut self, frame: &mut [u8], use_window: bool) {
        if let Some(bg_pixel) = self.bg_fifo.pop() {
            if self.pixels_to_discard > 0 {
                self.pixels_to_discard -= 1;
            } else {
                let obj_pixel = self.obj_piso.shift_out();

                let bg_color_index: u8;
                let bg_color: Color;

                // If BG is disabled, color 0 everywhere
                if !self.read_lcdc().is_bg_window_enabled() {
                    bg_color_index = 0;
                    bg_color = self.apply_background_palette(0);
                }
                else {
                    bg_color_index = bg_pixel.get_color_index();
                    bg_color = *bg_pixel.get_color();
                }

                let obj_color_index = obj_pixel.get_color_index();

                let final_color = if obj_color_index == 0 {
                    bg_color
                } else {
                    let priority = obj_pixel.get_priority();

                    if priority && bg_color_index != 0 {
                        bg_color
                    } else {
                        *obj_pixel.get_color()
                    }
                };

                let ly = self.ly as usize;

                let offset = (ly * WIN_SIZE_X + self.x) * 3; // * 3 for each pixels (3 bytes (RGB))
                self.set_pixel_color(frame, offset, final_color);

                self.x += 1;
            }
        }

    }


    fn step_pixel_fetcher(&mut self, use_window: bool) {
        let scy = self.read_scy();
        let scx = self.read_scx();

        let tile_pixels = self.pixel_fetcher.tick(&self.bus, &self.bg_fifo, self.ly, scx, scy, self.wly, &self.read_lcdc(), use_window);

        if let Some(pixels) = tile_pixels {
            for pixel in pixels {
                self.bg_fifo.push(pixel);
            }
        }
    }


    fn step_oam_fetcher(&mut self) {
        let height:u8 = if self.read_lcdc().is_obj_size_8x16() {
            16
        } else {
            8
        };

        if self.fetching_sprite {
            if let Some(index) = self.current_sprite_to_fetch {
                if let Some(sprite) = self.visible_sprites[index] {
                    let lcdc = self.read_lcdc();

                    self.fetching_sprite = !self.oam_fetcher.tick(
                        &self.bus,
                        &sprite,
                        &mut self.obj_piso,
                        self.ly,
                        &lcdc,
                        height,
                        self.x,
                    );


                    if !self.fetching_sprite {
                        self.visible_sprites[index] = None;
                    }
                }
            };
        }
        else {
            if !self.read_lcdc().is_obj_enabled() { return; }

            let lcdc = self.read_lcdc();

            for (index, sprite_opt) in self.visible_sprites.iter_mut().enumerate() {
                if let Some(sprite) = sprite_opt {

                    if sprite.x as usize <= self.x + 8 {
                        let spritex = sprite.x;
                        let selfx = self.x;

                        self.current_sprite_to_fetch = Some(index);
                        self.pixel_fetcher.reset_to_state_1();

                        self.fetching_sprite = !self.oam_fetcher.tick(
                            &self.bus,
                            sprite,
                            &mut self.obj_piso,
                            self.ly,
                            &lcdc,
                            height,
                            self.x,
                        );

                        if !self.fetching_sprite {
                            *sprite_opt = None;
                        }

                        break;
                    }
                }
            }
        }
    }


    fn mode_pixel_transfer(&mut self, image: &mut Arc<Mutex<Vec<u8>>>) -> bool {
        if self.ly < WIN_SIZE_Y as u8 {
            let wx = self.read_wx();

            let use_window = self.read_lcdc().is_window_enabled()
                && self.wy_equal_ly_condition_met
                && (self.x + 7 >= wx as usize);


            self.step_oam_fetcher();

            if !self.fetching_sprite {
                self.step_pixel_fetcher(use_window);
                let mut frame = image.lock().unwrap();
                self.handle_window_switch(use_window);
                self.push_pixel_to_screen(&mut frame, use_window);
            }
        }

        if self.x == 160 {
            self.lcd_status.update_ppu_mode(PpuMode::HBlank);
            self.write_stat_to_mmu();
        }

        false
    }


    fn mode_hblank(&mut self) -> bool {
        // End of scanline -> next one after 456 dots

        if self.dots >= SCANLINE_DOTS {
            self.dots -= SCANLINE_DOTS;
            
            let wy = self.read_wy();
            let wx = self.read_wx();

            if self.read_lcdc().is_window_enabled()
                && self.ly >= wy
                && wx <= 166 {
                self.wly += 1;
            }

            self.ly += 1;
            self.write_ly_to_mmu();

            self.check_lyc_equals_ly();

            // reset for newline
            // TODO proper reset function
            self.x = 0;
            self.bg_fifo.clear();
            self.obj_piso.reset();
            self.pixel_fetcher.reset_for_scanline();

            let scx = self.read_scx();

            self.pixels_to_discard = scx % 8;
            self.use_window = false;
            self.is_wx_glitch_happened = false;

            if self.ly >= WIN_SIZE_Y as u8 {
                self.lcd_status.update_ppu_mode(PpuMode::VBlank);
                self.write_stat_to_mmu();

                self.bus.write().unwrap().interrupts_request(Interrupt::VBlank);

                return true;
            } else {
                self.lcd_status.update_ppu_mode(PpuMode::OamSearch);
                self.write_stat_to_mmu();
            }
        }
        false
    }

    fn mode_vblank(&mut self) -> bool {
        if self.dots >= SCANLINE_DOTS {
            self.dots -= SCANLINE_DOTS;

            self.ly += 1;
            self.write_ly_to_mmu();

            self.check_lyc_equals_ly();

            if self.ly >= WIN_SIZE_Y as u8 + VBLANK_SIZE as u8 {
                self.ly = 0;
                self.write_ly_to_mmu();
               
                self.check_lyc_equals_ly();

                self.wly = 0;
                // TODO proper reset function
                self.x = 0;
                self.bg_fifo.clear();
                self.obj_piso.reset();
                self.pixel_fetcher.reset_for_scanline();

                let scx = self.read_scx();

                self.pixels_to_discard = scx % 8;
                self.use_window = false;
                self.is_wx_glitch_happened = false;
                self.wy_equal_ly_condition_met = false;

                self.lcd_status.update_ppu_mode(PpuMode::OamSearch);
                self.write_stat_to_mmu();
            }
        }
        false
    }

    pub fn tick(&mut self, image: &mut Arc<Mutex<Vec<u8>>>) -> bool {
        self.fetch_data_from_mmu();

        if !self.read_lcdc().is_ppu_enabled() {
            self.ly = 0;
            self.write_ly_to_mmu();

            self.dots = 0;
            self.lcd_status.update_ppu_mode(PpuMode::HBlank);
            self.write_stat_to_mmu();
            return false;
        }

        self.dots += 1;

        let wy = self.read_wy();

        if wy == self.ly { self.wy_equal_ly_condition_met = true; }

        let was_updated = match self.lcd_status.get_ppu_mode() {
            PpuMode::OamSearch => self.mode_oam_search(),
            PpuMode::PixelTransfer => self.mode_pixel_transfer(image),
            PpuMode::HBlank => self.mode_hblank(),
            PpuMode::VBlank => self.mode_vblank(),
        };

        was_updated
    }

    fn check_lyc_equals_ly(&mut self) {
        /*
            LYC == LY is an hardware condition:
                - update a flag in STAT
                - can trigger a LCD STAT interrupt
            It's used by many games to synchronize with scanline
        */
        let lyc_match = self.ly == self.read_lyc();
        self.lcd_status.set_lyc_equals_ly(lyc_match);
        self.write_stat_to_mmu();
        
        if self.lcd_status.get_lyc_equals_ly() {
            self.bus.write().unwrap().interrupts_request(Interrupt::LcdStat);
        }
    }

    pub fn fetch_data_from_mmu(&mut self) {
        let bus = self.bus.read().unwrap();
        self.lcd_status.update_from_byte(bus.read_byte(STAT_ADDR));
    }

    fn read_scy(&self) -> u8 {
        let bus = self.bus.read().unwrap();
        bus.read_byte(SCY_ADDR)
    }

    fn read_scx(&self) -> u8 {
        let bus = self.bus.read().unwrap();
        bus.read_byte(SCX_ADDR)
    }

    fn read_wy(&self) -> u8 {
        let bus = self.bus.read().unwrap();
        bus.read_byte(WY_ADDR)
    }

    fn read_wx(&self) -> u8 {
        let bus = self.bus.read().unwrap();
        bus.read_byte(WX_ADDR)
    }

    fn read_lyc(&self) -> u8 {
        let bus = self.bus.read().unwrap();
        bus.read_byte(LYC_ADDR)
    }

    fn read_lcdc(&self) -> LcdControl {
        let bus = self.bus.read().unwrap();
        let byte = bus.read_byte(LCD_CONTROL_ADDR);

        LcdControl::from_byte(byte)
    }

    fn write_ly_to_mmu(&mut self) {
        let mut bus = self.bus.write().unwrap();
        bus.write_byte(LY_ADDR, self.ly);
    }

    fn write_stat_to_mmu(&mut self) {
        let mut bus = self.bus.write().unwrap();
        bus.write_byte(STAT_ADDR, self.lcd_status.struct_to_byte());
    }
}
