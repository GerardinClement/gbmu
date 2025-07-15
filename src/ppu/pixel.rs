#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ppu::colors_palette::Color;

#[derive(Debug, Clone)]
pub struct Pixel {
    color: Color,
    palette: u8,
    sprite_priority: u8,
    background_priority: u8,
}

impl Pixel {
    pub fn new(color: Color, palette: u8, sprite_priority: u8, background_priority: u8) -> Self {
        Pixel {
            color,
            palette,
            sprite_priority,
            background_priority,
        }
    }

    pub fn get_color(&self) -> Color {
        self.color.clone()
    }

    pub fn get_palette(&self) -> u8 {
        self.palette
    }

    pub fn get_sprite_priority(&self) -> u8 {
        self.sprite_priority
    }

    pub fn get_background_priority(&self) -> u8 {
        self.background_priority
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: Color::White,
            palette: 0,
            sprite_priority: 0,
            background_priority: 0,
        }
    }
}
