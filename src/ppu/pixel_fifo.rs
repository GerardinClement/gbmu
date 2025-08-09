#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ppu::pixel::Pixel;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct PixelFifo {
    bg: VecDeque<Pixel>,
    y: u8,
    state: u8,
}

impl PixelFifo {
    pub fn new() -> Self {
        PixelFifo {
            bg: VecDeque::with_capacity(8),
            y: 0,
            state: 0,
        }
    }

    pub fn push(&mut self, pixel: Pixel) {
        self.bg.push_back(pixel);
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        self.bg.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.bg.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bg.len()
    }

    pub fn clear(&mut self) {
        self.bg.clear();
    }
}
