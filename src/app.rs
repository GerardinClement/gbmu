use crate::gameboy::GameBoy;
use crate::ppu::{self};

#[derive(Default)]
pub struct GameApp {
    gameboy: GameBoy,
    framebuffer: Vec<u8>,
}

impl GameApp {
    pub fn new(rom: Vec<u8>) -> Self {
        let gameboy = GameBoy::new(rom);
        println!("{}", gameboy.cpu);
        Self {
            gameboy,
            framebuffer: vec![0; ppu::WIN_SIZE_X * ppu::WIN_SIZE_Y * 4],
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
