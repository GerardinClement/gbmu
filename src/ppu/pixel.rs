#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ppu::colors_palette::Color;

#[derive(Debug, Clone)]
pub struct Pixel {
    color: Color,
    palette: u8,
    sprite_priority: u8,
    color_index: u8,
}

impl Pixel {
    pub fn new(color: Color, palette: u8, sprite_priority: u8, color_index: u8) -> Self {
        Pixel {
            color,
            palette,
            sprite_priority,
            color_index,
        }
    }

    pub fn get_color(&self) -> &Color {
        &self.color
    }

    pub fn get_palette(&self) -> u8 {
        self.palette
    }

    pub fn get_sprite_priority(&self) -> u8 {
        self.sprite_priority
    }

    pub fn get_color_index(&self) -> u8 {
        self.color_index
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: Color::White,
            palette: 0,
            sprite_priority: 0,
            color_index: 0,
        }
    }
}
