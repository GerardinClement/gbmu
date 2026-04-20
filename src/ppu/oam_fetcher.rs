#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::mmu::mbc::Mbc;
use crate::mmu::oam::Sprite;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::obj_piso::ObjPiso;

use std::sync::{Arc, RwLock};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum FetcherState {
    #[default]
    GetTileId = 0,
    GetLowData = 1,
    GetHighData = 2,
    PushPixel = 3,
}

#[derive(Default)]
pub struct OamFetcher {
    fetcher_state: FetcherState,
    tile_id: u8,
    tile_data_low: u8,
    tile_data_high: u8,
    dot_counter: u32,
}

impl OamFetcher {
     pub fn tick<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, sprite: &Sprite, piso: &ObjPiso, ly: u8, lcd_control: &LcdControl) -> bool {
        self.dot_counter += 1;

        if self.dot_counter % 2 == 0 {
            match self.fetcher_state {
                FetcherState::GetTileId => {
                    self.fetcher_state = FetcherState::GetLowData;

                    return false;
                },
                FetcherState::GetLowData => {
                    self.fetcher_state = FetcherState::GetHighData;
                    return false;
                },
                FetcherState::GetHighData => {
                    self.fetcher_state = FetcherState::PushPixel;

                    return false;
                },
                FetcherState::PushPixel => {
                    self.fetcher_state = FetcherState::GetTileId;

                    return false;
                }
            }
        }
        false
     }
}