/*
	enum ou struct the first 4 bytes. Maybe make booleans for flags
	instead of the byte 3.

	Writing data to OAM is special, have to use the DMA
	transfer method.


	Etape 1 - écrire au bon moment:
	- détecter les deux premiers bits du registre STAT pour
	savoir quel est le mode du PPU
	- Si le mode est 0 ou 1, je ne fais rien
	- Si le mode est 2, je bloque l'écriture sur l'OAM
	- Si le mode est 3, je bloque l'écriture sur l'OAM et la
	VRAM


	Etape 2 - structure de l'OAM:
	- un u8 pour chaque byte
	- Attribuer les 4 bytes pour les 40 sprites
	- Mapper les adresses

	Etape 3 - transfert DMA


	Sprite size (8×8 vs 8×16), et découpage en tuiles.

	DMA : blocage de 160 cycles, source basée sur le bank write de FF46.

	Scanline Mode 2 : collecte des 10 sprites, priorité par indice.

	Pixel‑level Mode 3 : tri par X, résolution des conflits via palette & priorité.

	Flags d’attributs (flip, palette, bank, priorité).

	Clipping partiel quand sprites sont hors des bords de l’écran.
*/


	// Etape 1 - écrire au bon moment:
	// - détecter les deux premiers bits du registre STAT pour
	// savoir quel est le mode du PPU
	// - Si le mode est 0 ou 1, je ne fais rien
	// - Si le mode est 2, je bloque l'écriture sur l'OAM
	// - Si le mode est 3, je bloque l'écriture sur l'OAM et la
	// VRAM

#[derive(Default, Copy)]
struct OamBytes {
	pub y: u8, // Y-position of the sprite
	pub x: u8, // X-position of the sprite
	pub tile: u8, // Tile index
	pub flags: u8, // palette, flip, priority
}

pub struct Oam {
	pub sprites: [OamBytes; 40],
}

impl Default for Oam {
	fn default() -> Self {
		Self { sprites: [OamBytes { y: 0xFF, x: 0xFF, tile: 0xFF, flags: 0xFF }; 40] }
	}
}

impl Oam {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn read(&self, addr:u16) -> u8 {
		0
	}

	pub fn write(&mut self, addr: u16, val: u8) {}
}