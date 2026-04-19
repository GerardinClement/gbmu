#![allow(unused_variables)]
#![allow(dead_code)]

// PISO stands for "Parallel In Serial Out"
// Since it's not a real FIFO in its behavior

use crate::ppu::pixel::Pixel;
use crate::ppu::colors_palette::Color;

#[derive(Default)]
pub struct ObjPiso {
    pixels: [Pixel; 8],
}

impl ObjPiso {
    pub fn new() -> Self {
        ObjPiso {
            pixels: [Pixel::default(); 8],
        }
    }

    pub fn merge(
        &mut self,
        tile_data_low: u8,
        tile_data_high: u8,
        sprite_x: u8,
        x_flip: bool,
        palette: u8,
        oam_index: u8,
        priority: bool,
    ) {
        for i in 0..8 {
            let pos = sprite_x as i16 + i as i16 - 8;

            // Check if the pixel is outside the fifo
            if pos < 0 || pos >= 8 {
                continue;
            }

            // Create the corresponding mask to extract the appropiate bit of tile data
            // given the current fifo index and whether the sprite is horizontally mirrored.
            let bit = if x_flip { i } else { 7 - i };
            let low = (tile_data_low >> bit) & 1;
            let high = (tile_data_high >> bit) & 1;

            // Merge the two bits to create the pixels's color number
            let color_index = low | (high << 1);

            // If the color number of the new pixel is transparent,
            // the pixel at the current fifo index is not modified
            if color_index == 0 {
                continue;
            }

            // Extract the current pixel at the current fifo index to compare it
            // against the new pixel
            let current_pixel = self.pixels[pos as usize];
            let current_pixel_color_index = current_pixel.get_color_index();

            // The current fifo index is set to the new pixel if the current pixel
            // is transparent
            if current_pixel_color_index == 0 {
                let color_pixel = (palette >> (color_index * 2)) & 0b11;

                self.pixels[pos as usize] = Pixel::new_obj(
                    Color::from_index(color_pixel),
                    color_index,
                    priority,
                    oam_index
                );
            }
        }
    }

    pub fn shift_out(&mut self) -> Pixel {
        let out = self.pixels[0];

        for i in 0..7 {
            self.pixels[i] = self.pixels[i + 1];
        }

        self.pixels[7] = Pixel::default();
        
        out
    }

    pub fn reset(&mut self) {
        self.pixels = [Pixel::default(); 8];
    }
}