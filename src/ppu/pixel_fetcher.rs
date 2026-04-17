#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::mmu::mbc::Mbc;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::pixel::Pixel;
use std::sync::{Arc, RwLock};

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
    pub fn tick<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, ly: u8, scx: u8, scy: u8, lcd_control: &LcdControl, use_window: bool) -> Option<[Pixel; 8]> {
        self.dot_counter += 1;
        if self.dot_counter % 2 == 0 {
            match self.fetcher_state {
                FetcherState::GetTileId => {
                    self.tile_id = self.get_tile_id(bus, ly, scx, scy, lcd_control, use_window);
                    self.fetcher_state = FetcherState::GetLowData;

                    return None
                },
                FetcherState::GetLowData => {
                    return None
                },
                FetcherState::GetHighData => return None,
                FetcherState::Sleep => return None,
                FetcherState::PushPixel => return None,
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

        let x = ((scx / 8) as u16 + self.fetcher_x as u16) & 0x1F; // mask to keep the 5 lowest bits
        let y: u16 = ((ly as u16 + scy as u16) / 8) & 0xFF;

        let offset = (y * 32 + x) as u16;

        let tile_number = bus
            .read()
            .unwrap()
            .read_byte(tilemap_base.start + offset);

        tile_number
    }
}