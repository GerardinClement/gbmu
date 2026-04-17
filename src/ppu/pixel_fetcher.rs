#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum FetcherState {
    #[default]
    GetTileId = 0,
    GetLowData = 1,
    GetHighData = 2,
    PushPixel = 3,
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