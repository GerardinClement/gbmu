#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::mmu::mbc::Mbc;
use crate::mmu::MemoryRegion;
use crate::mmu::oam::Sprite;
use crate::ppu::lcd_control::LcdControl;
use crate::ppu::obj_piso::ObjPiso;

use std::sync::{Arc, RwLock};

const OBP0_ADDR: u16 = 0xFF48; // Object Palette 0
const OBP1_ADDR: u16 = 0xFF49; // Object Palette 1
const VRAM: MemoryRegion = MemoryRegion::Vram; // Start of VRAM

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
    actual_sprite_line: usize,
}

impl OamFetcher {
    pub fn tick<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, sprite: &Sprite, piso: &mut ObjPiso, ly: u8, lcd_control: &LcdControl, height: u8, scanline_x: usize) -> bool {
        self.dot_counter = self.dot_counter.wrapping_add(1);

        if self.dot_counter % 2 == 0 {
            match self.fetcher_state {
                FetcherState::GetTileId => {
                    self.tile_id = self.get_tile_id(sprite, ly, height);
                    self.fetcher_state = FetcherState::GetLowData;

                    return false;
                },
                FetcherState::GetLowData => {
                    self.tile_data_low = self.get_tile_data_low(bus);
                    self.fetcher_state = FetcherState::GetHighData;
                    
                    return false;
                },
                FetcherState::GetHighData => {
                    self.tile_data_high = self.get_tile_data_high(bus);
                    self.fetcher_state = FetcherState::PushPixel;

                    return false;
                },
                FetcherState::PushPixel => {
                    self.push_pixel(bus, piso, sprite, scanline_x);
                    self.fetcher_state = FetcherState::GetTileId;

                    return true;
                }
            }
        }

        false
    }

    fn get_tile_id(&mut self, sprite: &Sprite, ly: u8, height: u8) -> u8 {
        let sprite_top: i16 = sprite.y as i16 - 16;
        let sprite_line = (ly as i16 - sprite_top) as usize;

        let y_flip = ((sprite.attributes >> 6) & 1) != 0;
        let actual_sprite_line = if y_flip { (height as usize - 1) - sprite_line } else { sprite_line };

        let tile_always_pair = if height == 16 { sprite.tile & 0xFE } else { sprite.tile };
        let tile_index = if height == 16 && actual_sprite_line >= 8 { tile_always_pair + 1 } else { tile_always_pair }; // offset if 8x16 because of end of tile index

        self.actual_sprite_line = actual_sprite_line;
        tile_index
    }

    fn get_tile_data_low<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>) -> u8 {
        let tile_address = VRAM.to_address()
            + (self.tile_id as u16 * 16)
            + (self.actual_sprite_line % 8 * 2) as u16;

        let data = bus.read().unwrap().ppu_read_byte(tile_address as u16);

        data
    }

    fn get_tile_data_high<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>) -> u8 {
        let tile_address = VRAM.to_address()
            + (self.tile_id as u16 * 16)
            + (self.actual_sprite_line % 8 * 2) as u16;

        let data = bus.read().unwrap().ppu_read_byte(tile_address as u16 + 1);

        data
    }

    fn extract_attributes(&self, attributes: u8) -> (bool, bool, bool, bool) {
        (
            ((attributes >> 7) & 1) != 0,
            ((attributes >> 6) & 1) != 0,
            ((attributes >> 5) & 1) != 0,
            ((attributes >> 4) & 1) != 0,
        )
    }

    fn push_pixel<T: Mbc>(&mut self, bus: &Arc<RwLock<Mmu<T>>>, piso: &mut ObjPiso, sprite: &Sprite, scanline_x: usize) {
        let (priority, _, x_flip, palette_attribute) = self.extract_attributes(sprite.attributes);

        let palette_addr = if palette_attribute { OBP1_ADDR } else { OBP0_ADDR };
        let palette = bus.read().unwrap().ppu_read_byte(palette_addr);

        piso.merge(self.tile_data_low, self.tile_data_high, sprite.x, x_flip, palette, sprite.oam_index, priority, scanline_x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mmu::Mmu;
    use crate::mmu::mbc::RomOnly;

    use std::sync::{Arc, RwLock};

    fn setup_bus() -> Arc<RwLock<Mmu<RomOnly>>> {
        Arc::new(RwLock::new(
            Mmu::<RomOnly>::new(&[]).unwrap()
        ))
    }

    #[test]
    fn test_state_progression() {
        let mut fetcher = OamFetcher::default();

        let bus = setup_bus();
        let sprite = Sprite { y: 16, x: 8, tile: 0, oam_index: 0, attributes: 0 };
        let mut piso = ObjPiso::new();
        let lcd = LcdControl::default();

        // tick 1 -> nothing (odd dot)
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), false);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetTileId);

        // tick 2 -> GetTileId
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), false);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetLowData);

        // tick 4 -> GetLowData
        fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetLowData);
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), false);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetHighData);

        // tick 6 -> GetHighData
        fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetHighData);
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), false);
        assert_eq!(fetcher.fetcher_state, FetcherState::PushPixel);

        // tick 8 -> PushPixel → return false
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), false);
        assert_eq!(fetcher.fetcher_state, FetcherState::PushPixel);
        assert_eq!(fetcher.tick(&bus, &sprite, &mut piso, 0, &lcd, 8, 0), true);
        assert_eq!(fetcher.fetcher_state, FetcherState::GetTileId);
    }

    #[test]
    fn test_sprite_line_no_flip() {
        let mut fetcher = OamFetcher::default();

        let sprite = Sprite {
            y: 16 + 5, // sprite begin at y=5
            x: 0,
            tile: 3,
            oam_index: 0,
            attributes: 0,
        };

        let line = fetcher.get_tile_id(&sprite, 5, 8);

        assert_eq!(fetcher.actual_sprite_line, 0);
    }

    #[test]
    fn test_sprite_line_with_y_flip() {
        let mut fetcher = OamFetcher::default();

        let sprite = Sprite {
            y: 16,
            tile: 0,
            attributes: 0b0100_0000, // y_flip
            ..Default::default()
        };

        fetcher.get_tile_id(&sprite, 0, 8);

        // reversed line -> 7 instead of 0
        assert_eq!(fetcher.actual_sprite_line, 7);
    }

    #[test]
    fn test_sprite_8x16_tile_selection() {
        let mut fetcher = OamFetcher::default();

        let sprite = Sprite {
            y: 16,
            tile: 5,
            attributes: 0,
            ..Default::default()
        };

        // line in second hald
        let tile = fetcher.get_tile_id(&sprite, 10, 16);

        // tile have to be pair + 1
        assert_eq!(tile, (5 & 0xFE) + 1);
    }

    #[test]
    fn test_sprite_8x16_with_y_flip() {
        let mut fetcher = OamFetcher::default();

        let sprite = Sprite {
            y: 16,
            x: 0,
            tile: 4,
            oam_index: 0,
            attributes: 0b0100_0000,
        };

        let tile = fetcher.get_tile_id(&sprite, 12, 16);
        let base_tile = 4 & 0xFE;

        assert_eq!(
            tile,
            base_tile,
            "Y flip in 8x16 sprite should reverse tile selection"
        );
    }
}
