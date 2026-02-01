#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ppu::colors_palette::Color;

#[derive(Debug, Clone)]
pub struct Pixel {
    color: Color,
    is_sprite: bool,
    color_index: u8,
}

impl Pixel {
    pub fn new(color: Color, is_sprite: bool, color_index: u8) -> Self {
        Pixel {
            color,
            is_sprite,
            color_index,
        }
    }

    pub fn get_color(&self) -> &Color {
        &self.color
    }

    pub fn get_is_sprite(&self) -> bool {
        self.is_sprite
    }

    pub fn get_color_index(&self) -> u8 {
        self.color_index
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: Color::White,
            is_sprite: false,
            color_index: 0,
        }
    }
}
