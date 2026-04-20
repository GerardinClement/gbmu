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


// tests

// merge replace a transparent pixel

// merge does not replace a transparent pixel

// merge ignore the pixels in overscan (pos < 0)

// shift out shift correctly and insert a transparent pixel

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::pixel::Pixel;

    // helper
    fn make_pixel(color_index: u8) -> Pixel {
        Pixel::new_obj(
            Color::from_index(color_index),
            color_index,
            false,
            0,
        )
    }

    #[test]
    fn merge_replaces_transparent_pixel() {
        let mut piso = ObjPiso::new();

        // sprite x = 8 -> aligned on FIFO[0]
        piso.merge(
            0b1000_0000, // bit 7 = 1 -> pixel visible for i=0
            0,
            8,
            false,
            0b1110_0100,
            0,
            false,
        );

        assert_ne!(piso.pixels[0].get_color_index(), 0, "The pixel should not be transparent anymore");
    }

    #[test]
    fn merge_does_not_replace_existing_pixel() {
        let mut piso = ObjPiso::new();

        piso.pixels[0] = make_pixel(2);

        // Try to put a pixel at the same place
        piso.merge(
            0b1000_0000,
            0,
            8,
            false,
            0b1110_0100,
            1,
            false,
        );

        assert_eq!(piso.pixels[0].get_color_index(), 2, "The pixel should not change.");
    }

    #[test]
    fn merge_ignores_pixels_left_overscan() {
        let mut piso = ObjPiso::new();

        // sprite_x = 5 -> some pixels will have pos < 0
        piso.merge(
            0xFF, // all actives pixels
            0,
            5,
            false,
            0b1110_0100,
            0,
            false,
        );
        assert_ne!(piso.pixels[0].get_color_index(), 0, "The last slots should not be transparents");
        assert_ne!(piso.pixels[1].get_color_index(), 0, "The last slots should not be transparents");
        assert_ne!(piso.pixels[2].get_color_index(), 0, "The last slots should not be transparents");
        assert_ne!(piso.pixels[3].get_color_index(), 0, "The last slots should not be transparents");
        assert_ne!(piso.pixels[4].get_color_index(), 0, "The last slots should not be transparents");

        assert_eq!(piso.pixels[5].get_color_index(), 0, "The first slots have to stay transparents");
        assert_eq!(piso.pixels[6].get_color_index(), 0, "The first slots have to stay transparents");
        assert_eq!(piso.pixels[7].get_color_index(), 0, "The first slots have to stay transparents");
    }

    #[test]
    fn merge_ignores_transparent_pixels() {
        let mut piso = ObjPiso::new();

        // tile data = 0 -> all pixels transparents
        piso.merge(
            0,
            0,
            8,
            false,
            0b1110_0100,
            0,
            false,
        );

        for i in 0..8 {
            assert_eq!(piso.pixels[i].get_color_index(), 0, "The pixel is not transparent.");
        }
    }

    #[test]
    fn shift_out_shifts_correctly() {
        let mut piso = ObjPiso::new();

        for i in 0..8 {
            piso.pixels[i] = make_pixel((i % 4) as u8);
        }

        let out = piso.shift_out();

        assert_eq!(out.get_color_index(), 0, "The first pixel should shift out of the piso");

        // shift
        for i in 0..7 {
            assert_eq!(piso.pixels[i].get_color_index(), ((i + 1) % 4) as u8, "A pixel didn't shift.");
        }

        assert_eq!(piso.pixels[7].get_color_index(), 0, "The last pixel should be transparent");
    }

    #[test]
    fn reset_clears_fifo() {
        let mut piso = ObjPiso::new();

        piso.pixels[3] = make_pixel(2);

        piso.reset();

        for i in 0..8 {
            assert_eq!(piso.pixels[i].get_color_index(), 0, "The reset function hasn't erased all pixels.");
        }
    }


}