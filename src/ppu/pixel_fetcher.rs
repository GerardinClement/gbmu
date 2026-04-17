#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::mmu::mbc::Mbc;
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
    position_counter: u8,
}

impl PixelFetcher {
    pub fn tick<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, ly: u8, scx: u8, scy: u8, use_window: bool) -> Option<[Pixel; 8]> {
        if self.dot_counter % 2 == 0 {
            match self.fetcher_state {
                FetcherState::GetTileId => return None,
                FetcherState::GetLowData => return None,
                FetcherState::GetHighData => return None,
                FetcherState::Sleep => return None,
                FetcherState::PushPixel => return None,
            }
        } else {
            None
        }
    }
}