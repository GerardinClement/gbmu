const OAM_BEGINNING: u16 = 0xFE00;

#[derive(Default, Clone, Copy)]
pub struct Sprite {
	pub y: u8, // Y-position of the sprite
	pub x: u8, // X-position of the sprite
	pub tile: u8, // Tile index
	pub attributes: u8, // bit 7: Priority, bit 6: Y flip, bit 5: X flip, bit 4: Palette, bit 3-0: unused for DMG
}

impl Sprite {
    pub fn is_visible(&self, ly: u8, height: u8) -> bool {
        if ly >= 144 { // out of the screen
            return false;
        }

        let sprite_top: i16 = self.y as i16 - 16;
        let sprite_bottom: i16 = sprite_top + height as i16;
        let ly: i16 = ly as i16;

        ly >= sprite_top && ly < sprite_bottom
    }
}

pub struct Oam {
	pub sprites: [Sprite; 40],
}

impl Default for Oam {
	fn default() -> Self {
		Self { sprites: [Sprite { y: 0xFF, x: 0xFF, tile: 0xFF, attributes: 0xFF }; 40] }
	}
}

impl Oam {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn read(&self, addr:u16) -> u8 {
		let sprite = ((addr - OAM_BEGINNING) / 4) as usize;
		let byte = ((addr - OAM_BEGINNING) % 4) as usize;

		match byte {
			0 => self.sprites[sprite].y,
			1 => self.sprites[sprite].x,
			2 => self.sprites[sprite].tile,
			3 => self.sprites[sprite].attributes,
			_ => 0,
		}
	}

	pub fn write(&mut self, addr: u16, val: u8) {
		let sprite = ((addr - OAM_BEGINNING) / 4) as usize;
		let byte = ((addr - OAM_BEGINNING) % 4) as usize;

		match byte {
			0 => self.sprites[sprite].y = val,
			1 => self.sprites[sprite].x = val,
			2 => self.sprites[sprite].tile = val,
			3 => self.sprites[sprite].attributes = val,
			_ => return,
		}
	}
}


#[cfg(test)]
mod tests {
    use super::Oam;
    use super::Sprite;

    #[test]
    fn test_write_sprite_0_y_position() {
        let mut oam = Oam::new();
        oam.write(0xFE00, 0x50);
        assert_eq!(oam.sprites[0].y, 0x50);
    }

    #[test]
    fn test_write_sprite_0_x_position() {
        let mut oam = Oam::new();
        oam.write(0xFE01, 0x30);
        assert_eq!(oam.sprites[0].x, 0x30);
    }

    #[test]
    fn test_write_sprite_0_tile() {
        let mut oam = Oam::new();
        oam.write(0xFE02, 0x42);
        assert_eq!(oam.sprites[0].tile, 0x42);
    }

    #[test]
    fn test_write_sprite_0_attributes() {
        let mut oam = Oam::new();
        oam.write(0xFE03, 0xAB);
        assert_eq!(oam.sprites[0].attributes, 0xAB);
    }

    #[test]
    fn test_write_sprite_5_all_bytes() {
        let mut oam = Oam::new();
        oam.write(0xFE14, 100); // Sprite 5, Y
        oam.write(0xFE15, 80);  // Sprite 5, X
        oam.write(0xFE16, 25);  // Sprite 5, tile
        oam.write(0xFE17, 0x20); // Sprite 5, attributes
        
        assert_eq!(oam.sprites[5].y, 100);
        assert_eq!(oam.sprites[5].x, 80);
        assert_eq!(oam.sprites[5].tile, 25);
        assert_eq!(oam.sprites[5].attributes, 0x20);
    }

    #[test]
    fn test_write_sprite_39_last_sprite() {
        let mut oam = Oam::new();
        oam.write(0xFE9C, 144); // Sprite 39, Y
        oam.write(0xFE9D, 160); // Sprite 39, X
        oam.write(0xFE9E, 99);  // Sprite 39, tile
        oam.write(0xFE9F, 0xFF); // Sprite 39, attributes
        
        assert_eq!(oam.sprites[39].y, 144);
        assert_eq!(oam.sprites[39].x, 160);
        assert_eq!(oam.sprites[39].tile, 99);
        assert_eq!(oam.sprites[39].attributes, 0xFF);
    }

    #[test]
    fn test_read_sprite_0_y_position() {
        let mut oam = Oam::new();
        oam.sprites[0].y = 0x88;
        assert_eq!(oam.read(0xFE00), 0x88);
    }

    #[test]
    fn test_read_sprite_6_y_position() {
        let mut oam = Oam::new();
        oam.sprites[6].y = 0x88;
        assert_eq!(oam.read(0xFE18), 0x88);
    }
	
    #[test]
    fn test_read_sprite_0_x_position() {
        let mut oam = Oam::new();
        oam.sprites[0].x = 0x77;
        assert_eq!(oam.read(0xFE01), 0x77);
    }

    #[test]
    fn test_read_sprite_0_tile() {
        let mut oam = Oam::new();
        oam.sprites[0].tile = 0x12;
        assert_eq!(oam.read(0xFE02), 0x12);
    }

    #[test]
    fn test_read_sprite_0_attributes() {
        let mut oam = Oam::new();
        oam.sprites[0].attributes = 0xCD;
        assert_eq!(oam.read(0xFE03), 0xCD);
    }

    #[test]
    fn test_read_write_roundtrip() {
        let mut oam = Oam::new();
        
        // Write values
        oam.write(0xFE20, 55);  // Sprite 8, Y
        oam.write(0xFE21, 99);  // Sprite 8, X
        oam.write(0xFE22, 123); // Sprite 8, tile
        oam.write(0xFE23, 0xAB); // Sprite 8, attributes
        
        // Read them back
        assert_eq!(oam.read(0xFE20), 55);
        assert_eq!(oam.read(0xFE21), 99);
        assert_eq!(oam.read(0xFE22), 123);
        assert_eq!(oam.read(0xFE23), 0xAB);
    }

    #[test]
    fn test_multiple_sprites_independence() {
        let mut oam = Oam::new();
        
        // Write to sprite 0
        oam.write(0xFE00, 10);
        oam.write(0xFE01, 20);
        
        // Write to sprite 1
        oam.write(0xFE04, 30);
        oam.write(0xFE05, 40);
        
        // Verify sprite 0
        assert_eq!(oam.read(0xFE00), 10);
        assert_eq!(oam.read(0xFE01), 20);
        
        // Verify sprite 1
        assert_eq!(oam.read(0xFE04), 30);
        assert_eq!(oam.read(0xFE05), 40);
    } 
    
    /*
        is sprite visible tests
    */

    #[test]
        fn test_sprite_visible_middle_of_screen() {
            let sprite = Sprite { y: 50, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 50 - 16 = 34
            // Visible sur lignes 34 à 41 (8 pixels de haut)
            assert!(sprite.is_visible(34, 8));
            assert!(sprite.is_visible(37, 8));
            assert!(sprite.is_visible(41, 8));
        }

        #[test]
        fn test_sprite_not_visible_before() {
            let sprite = Sprite { y: 50, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 34, donc pas visible sur ligne 33
            assert!(!sprite.is_visible(33, 8));
            assert!(!sprite.is_visible(20, 8));
            assert!(!sprite.is_visible(0, 8));
        }

        #[test]
        fn test_sprite_not_visible_after() {
            let sprite = Sprite { y: 50, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 34, hauteur 8, donc pas visible après ligne 41
            assert!(!sprite.is_visible(42, 8));
            assert!(!sprite.is_visible(100, 8));
            assert!(!sprite.is_visible(143, 8));
        }

        #[test]
        fn test_sprite_at_top_edge() {
            let sprite = Sprite { y: 16, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 16 - 16 = 0
            // Visible sur lignes 0 à 7
            assert!(sprite.is_visible(0, 8));
            assert!(sprite.is_visible(7, 8));
            assert!(!sprite.is_visible(8, 8));
        }

        #[test]
        fn test_sprite_partially_offscreen_top() {
            let sprite = Sprite { y: 10, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 10 - 16 = -6 (partiellement hors écran en haut)
            // Visible sur lignes 0 à 1 (les 2 dernières lignes du sprite)
            assert!(sprite.is_visible(0, 8));
            assert!(sprite.is_visible(1, 8));
            assert!(!sprite.is_visible(2, 8));
        }

        #[test]
        fn test_sprite_completely_offscreen_top() {
            let sprite = Sprite { y: 5, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 5 - 16 = -11 (complètement hors écran)
            assert!(!sprite.is_visible(0, 8));
            assert!(!sprite.is_visible(10, 8));
        }

        #[test]
        fn test_sprite_at_bottom_edge() {
            let sprite = Sprite { y: 160, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 160 - 16 = 144
            // Ligne 144 et au-delà sont hors de l'écran visible (écran = 0-143)
            assert!(!sprite.is_visible(143, 8));
            assert!(!sprite.is_visible(144, 8));
        }

        #[test]
        fn test_sprite_8x16_mode() {
            let sprite = Sprite { y: 50, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 34
            // En mode 8x16, visible sur lignes 34 à 49 (16 pixels de haut)
            assert!(sprite.is_visible(34, 16));
            assert!(sprite.is_visible(40, 16));
            assert!(sprite.is_visible(49, 16));
            assert!(!sprite.is_visible(50, 16));
            assert!(!sprite.is_visible(33, 16));
        }

        #[test]
        fn test_edge_case_y_equals_zero() {
            let sprite = Sprite { y: 0, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 0 - 16 = -16 (complètement hors écran)
            assert!(!sprite.is_visible(0, 8));
            assert!(!sprite.is_visible(100, 8));
        }

        #[test]
        fn test_edge_case_y_equals_255() {
            let sprite = Sprite { y: 255, x: 80, tile: 0, attributes: 0 };
            // Position réelle = 255 - 16 = 239 (très en bas, hors écran)
            assert!(!sprite.is_visible(143, 8));
            assert!(!sprite.is_visible(0, 8));
        }
}
