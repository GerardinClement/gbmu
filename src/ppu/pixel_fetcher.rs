#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::mmu::mbc::Mbc;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::pixel::Pixel;
use crate::ppu::colors_palette::Color;
use crate::ppu::pixel_fifo::PixelFifo;
use std::sync::{Arc, RwLock};

const BGP_ADDR: u16 = 0xFF47; // Background Palette
const TILE_DATA_1_START: u16 = 0x8000;
const TILE_DATA_0_START: u16 = 0x8800;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum FetcherState {
    #[default]
    GetTileId = 0,
    GetLowData = 1,
    GetHighData = 2,
    Sleep = 3,
    PushPixel = 4,
}

#[derive(Default)]
pub struct PixelFetcher {
    fetcher_state: FetcherState,
    tile_id: u8,
    tile_data_low: u8,
    tile_data_high: u8,
    dot_counter: u32,
    fetcher_x: u8,
}

impl PixelFetcher {
    pub fn tick<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, fifo: &PixelFifo, ly: u8, scx: u8, scy: u8, lcd_control: &LcdControl, use_window: bool) -> Option<[Pixel; 8]> {
        self.dot_counter += 1;

        if self.fetcher_state == FetcherState::PushPixel && fifo.is_empty() {
            let tile: Option<[Pixel; 8]> = self.push_pixel(bus);

            self.fetcher_x += 1;
            self.fetcher_state = FetcherState::GetTileId;

            tile
        } else if self.dot_counter % 2 == 0 {
            match self.fetcher_state {
                FetcherState::GetTileId => {
                    self.tile_id = self.get_tile_id(bus, ly, scx, scy, lcd_control, use_window);
                    self.fetcher_state = FetcherState::GetLowData;

                    return None
                },
                FetcherState::GetLowData => {
                    self.tile_data_low = self.get_tile_data_low(bus, ly, scy, lcd_control);
                    self.fetcher_state = FetcherState::GetHighData;

                    return None
                },
                FetcherState::GetHighData => {
                    self.tile_data_high = self.get_tile_data_high(bus, ly, scy, lcd_control);
                    self.fetcher_state = FetcherState::Sleep;

                    return None
                },
                FetcherState::Sleep => {
                    self.fetcher_state = FetcherState::PushPixel;

                    return None
                },
                FetcherState::PushPixel => { return None; },
            }
        } else {
            None
        }
    }

    fn get_tile_id<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, ly: u8, scx: u8, scy: u8, lcd_control: &LcdControl, use_window: bool) -> u8 {
        let tilemap_base: std::ops::Range<u16> = if use_window {
            lcd_control.window_tile_map_area()
        } else {
            lcd_control.bg_tile_map_area()
        };

        let x: u16 = ((scx / 8) as u16 + self.fetcher_x as u16) & 0x1F; // mask to keep the 5 lowest bits
        let y: u16 = ((ly as u16 + scy as u16) / 8) & 0xFF;

        let offset = (y * 32 + x) as u16;

        let tile_number = bus
            .read()
            .unwrap()
            .read_byte(tilemap_base.start + offset);

        tile_number
    }

    fn get_tile_data_low<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, ly: u8, scy: u8, lcd_control: &LcdControl) -> u8 {
        let correct_byte = ((ly as u16 + scy as u16) % 8) * 2;

        if lcd_control.bg_window_tile_data_area().start == TILE_DATA_1_START {
            let tilemap_base = TILE_DATA_1_START + (self.tile_id as u16) * 16;

            let tile_low = bus
                .read()
                .unwrap()
                .read_byte(tilemap_base + correct_byte);

            tile_low
            
        } else if lcd_control.bg_window_tile_data_area().start == TILE_DATA_0_START {
            let base = 0x9000u16;
            let offset = (self.tile_id as i8) as i16 * 16;
            let tilemap_base = base.wrapping_add_signed(offset);

            let tile_low = bus
                .read()
                .unwrap()
                .read_byte(tilemap_base + correct_byte);

            tile_low
        } else {
            unreachable!()
        }
    }


    fn get_tile_data_high<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, ly: u8, scy: u8, lcd_control: &LcdControl) -> u8 {
        let correct_byte = (((ly as u16 + scy as u16) % 8) * 2) + 1;

        if lcd_control.bg_window_tile_data_area().start == TILE_DATA_1_START {
            let tilemap_base = TILE_DATA_1_START + (self.tile_id as u16) * 16;

            let tile_low = bus
                .read()
                .unwrap()
                .read_byte(tilemap_base + correct_byte);

            tile_low
            
        } else if lcd_control.bg_window_tile_data_area().start == TILE_DATA_0_START {
            let base = 0x9000u16;
            let offset = (self.tile_id as i8) as i16 * 16;
            let tilemap_base = base.wrapping_add_signed(offset);

            let tile_low = bus
                .read()
                .unwrap()
                .read_byte(tilemap_base + correct_byte);

            tile_low
        } else {
            unreachable!()
        }
    }

    fn apply_background_palette<T: Mbc>(&self, bus: &Arc<RwLock<Mmu<T>>>, color_index: u8) -> Color {
        let palette = bus.read().unwrap().read_byte(BGP_ADDR);

        let index = (palette >> (color_index * 2)) & 0b11;

        Color::from_index(index)
    }

    fn push_pixel<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>) -> Option<[Pixel; 8]> {
        let mut tile_pixels = [Pixel::default(); 8];

        for i in 0..8 {
            let bit_index = 7 - i;

            let low_weight_bit = (self.tile_data_low >> bit_index) & 1;
            let high_weight_bit = (self.tile_data_high >> bit_index) & 1;

            let color_index = (high_weight_bit << 1) | low_weight_bit;
            let bgp = self.apply_background_palette(bus, color_index);

            let pixel = Pixel::new(bgp, false, color_index);
            
            tile_pixels[i as usize] = pixel;
        }

        Some(tile_pixels)
    }
}