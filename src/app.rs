#![allow(unreachable_code)]

use crate::{DebugCommandQueries, DebugResponse, WatchedAdresses, gameboy::GameBoy};
use crate::ppu;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Default)]
pub struct GameApp {
    gameboy: GameBoy,
    debug_receiver: Receiver<DebugCommandQueries>,
    debug_sender: Sender<DebugResponse>,
    is_step_mode: bool,
    nb_next_intruction: u8,
    is_sending_registers: bool,
    watched_adress: WatchedAdresses,
}

impl GameApp {
    pub fn new(rom: Vec<u8>) -> Self {
        let gameboy = GameBoy::new(rom);
        println!("{}", gameboy.cpu);
        Self {
            gameboy,
            debug_receiver: receiver,
            debug_sender: sender,
            is_step_mode: false,
            is_debug_mode: global_bool,
            is_sending_registers: false,
            nb_next_intruction: 0,
            watched_adress: WatchedAdresses {
                addresses_n_values: Vec::new(),
            },
        }
    }

    pub fn update(&mut self) -> Option<Vec<u8>> {
        let rgb_frame = self.gameboy.run_frame();
        Some(Self::rgb_to_rgba(&rgb_frame))
    }

    fn rgb_to_rgba(rgb_frame: &[u8]) -> Vec<u8> {
        let mut rgba_frame = Vec::with_capacity(ppu::WIN_SIZE_X * ppu::WIN_SIZE_Y * 4);
        for chunk in rgb_frame.chunks(3) {
            rgba_frame.extend_from_slice(chunk);
            rgba_frame.push(255);
        }
        rgba_frame
    }
}
