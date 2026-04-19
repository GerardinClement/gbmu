#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ppu::colors_palette::Color;

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    color: Color,
    is_sprite: bool,
    color_index: u8,
    priority: bool,
    oam_index: u8,
}

impl Pixel {
    pub fn new_bg(color: Color, color_index: u8) -> Self {
        let priority = false;
        let oam_index = 0;
        let is_sprite = false;

        Pixel {
            color,
            is_sprite,
            color_index,
            priority,
            oam_index,
        }
    }

    pub fn new_obj(color: Color, color_index: u8, priority: bool, oam_index: u8) -> Self {
        let is_sprite = true;

        Pixel {
            color,
            is_sprite,
            color_index,
            priority,
            oam_index,
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

    pub fn get_priority(&self) -> bool {
        self.priority
    }

    pub fn get_oam_index(&self) -> u8 {
        self.oam_index
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: Color::White,
            is_sprite: false,
            color_index: 0,
            priority: false,
            oam_index: 0,
        }
    }
}