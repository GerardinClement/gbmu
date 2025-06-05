
## Game Boy Memory Map (0x0000–0xFFFF)


|Range           |	Size     |               Name            |	Purpose / Notes                                     |
|----------------|-----------|-------------------------------|------------------------------------------------------|
|0x0000–0x3FFF	 |   16 KiB	 |   [ROM Bank 0](#1-rom-bank-0-0x0000-0x3fff)	                 |  Fixed startup code, reset/jump vectors, cartridge header (title, MBC type, ROM/RAM size, checksum). Read-only.                                                        |
|0x4000–0x7FFF	 |   16 KiB	 |   [ROM Bank 1–N](#2-switchable-rom-0x4000-0x7fff)                |	Switchable game code/data via MBC (bank-switching). |
|0x8000–0x9FFF	 |   8 KiB	 |   [Video RAM (VRAM)](#3-vram-0x8000-0x9fff)            |	Tile pixel data (0x8000–0x8FFF), tile maps (0x9800–0x9BFF & 0x9C00–0x9FFF).                    |
|0xA000–0xBFFF	 |   8 KiB	 |   [External RAM](#4-external-ram-0xa000-0xbfff)	             |  Cartridge RAM (battery-backed save data). May be banked by MBC.                               |
|0xC000–0xCFFF	 |   4 KiB	 |   [Work RAM Bank 0 (WRAM0)](#5-work-ram-0xc000-0xdfff)     |	Main Cpu RAM.                                       |
|0xD000–0xDFFF	 |   4 KiB	 |   [Work RAM Bank 1–7 (WRAM1–7)](#5-work-ram-0xc000-0xdfff) |	Additional banks on CGB (color mode); on DMG always bank 1.                                      |
|0xE000–0xFDFF	 |   7.5 KiB |	[Echo RAM](#6-echo-ram-0xe000-0xfdff)                     |	Mirror of 0xC000–0xDDFF. Reads/writes act exactly like WRAM.                                  |
|0xFE00–0xFE9F	 |   160 B	 |   [OAM (Sprite Attribute Table)](#7-oam-0xfe00-0xfe9f)|	40 sprites × 4 B each: Y, X, tile#, flags.          |
|0xFEA0–0xFEFF	 |   96 B	 |   [Unusable](#8-unusable-0xfea0-0xfeff)                  |	Forbidden. Returns open-bus or $FF.                 |
|0xFF00–0xFF7F	 |   128 B	 |   [I/O Registers](#9-io-registers-0xff00-0xff7f)               |	Joypad, serial, timer/divider, interrupts, audio, LCD control, DMA, CGB extras. Reads/writes dispatch to hardware modules.          |
|0xFF80–0xFFFE	 |   127 B	 |   [High RAM (HRAM)](#10-high-ram-0xff80-0xfffe)             |	Fast “zero-page” RAM for critical data.             |
|0xFFFF–0xFFFF	 |   1 B	 |   [Interrupt-Enable (IE)](#11-interrupt-enable-0xffff)       |	Bitmask enabling V-Blank, LCD STAT, Timer, Serial, Joypad interrupts.                          |

## Region-by-Region Breakdown

#### 1. ROM Bank 0 (`0x0000-0x3FFF`)
- Cpu’s first instructions and RST vectors.
- **Cartridge header** (`0x0100–0x014F`): title, CGB flag, MBC type, ROM/RAM sizes, checksum.
- Writes here become MBC commands (do not overwrite code).

#### 2. Switchable ROM (`0x4000-0x7FFF`)
- Writes to (`0x2000-0x3FFF`), your MBC will pick which 16 KiB bank appears here.
- **Rust tip**: store `Vec<Vec<u8>>` of 16 KiB banks and `copy_from_slice()` into `data[0x4000..0x8000]`.

#### 3. VRAM (`0x8000-0x9FFF`)
- **Tile data** (`0x8000–0x8FFF`): 384 tiles × 16 B (2 bpp).
- **Tile maps** (`0x9800–0x9BFF`, `0x9C00–0x9FFF`): 32 × 32 entries.
- **CGB**: write `0xFF4F` to switch VRAM bank (VBK=0/1).

#### 4. External RAM (`0xA000-0xBFFF`)
- Cartridge-provided RAM (`0xA000–0xBFFF`), usually battery-backed.
- Bank-switched by MBC registers (`0x0000–0x7FFF`).
- Absent RAM → reads return $FF, writes ignored.

#### 5. Work RAM (`0xC000-0xDFFF`)
- **Bank 0**: `0xC000–0xCFFF`.
- **Banks 1–7**: `0xD000–0xDFFF` (select via `0xFF70` on CGB; on DMG always bank 1).

#### 6. Echo RAM (`0xE000-0xFDFF`)
- Simple mirror of `0xC000–0xDDFF`.
- **Implementation**: if `addr >= 0xE000 && addr <= 0xFDFF`, do `read_byte(addr - 0x2000)` (and same for write).

#### 7. OAM (`0xFE00-0xFE9F`)
- **Sprite Attribute Table**: 40 sprites × 4 bytes:
    1. Y-position
    2. X-position
    3. Tile index
    4. Flags (palette, flip, priority)

- **DMA**: a write to `0xFF46` starts a quick 160 B copy from `val<<8` → `0xFE00–0xFE9F`.

#### 8. Unusable (`0xFEA0-0xFEFF`)
- Forbidden. Games must not use this.
- On DMG/CGB reads usually return `$FF` or open bus; writes do nothing.

#### 9. I/O Registers (`0xFF00-0xFF7F`)
Each address here is a hardware register, not plain RAM. You’ll dispatch reads/writes to your various subsystems:

| Addr      | Name           | Purpose                                             |
| --------- | -------------- | --------------------------------------------------- |
| `FF00`    | **JOYP**       | Joypad input select + state                         |
| `FF01–02` | **Serial**     | Serial data transfer                                |
| `FF04–07` | **Timer**      | DIV, TIMA, TMA, TAC                                 |
| `FF0F`    | **IF**         | Interrupt Flag                                      |
| `FF10–3F` | **APU**        | Sound registers                                     |
| `FF40–4B` | **LCD**        | LCDC, STAT, SCY, SCX, LY, LYC, DMA, BGP, OBP0, OBP1 |
| `FF46`    | **DMA**        | OAM DMA transfer trigger                            |
| `FF4F`    | **VBK**        | VRAM bank select (CGB only)                         |
| `FF50`    | **BOOT**       | Disable BOOT ROM when set ≠ 0 (DMG)                 |
| `FF51–55` | **HDMA**       | HBlank/CGB DMA control                              |
| `FF68–6B` | **BG/OBJ Pal** | CGB palette data                                    |
| `FF70`    | **SVBK**       | WRAM bank select (CGB only)                         |
| …others…  |                | Link, sound, LCD window, OAM corruption, etc.       |

#### JOYP (0xFF00) bit layout

| Bit | P15=0 (btn) | P14=0 (dir) |
|:---:|:-----------:|:-----------:|
| 3   | Start       | Down        |
| 2   | Select      | Up          |
| 1   | B           | Left        |
| 0   | A           | Right       |

_Write `0b0010_0000` to select buttons (P15=0) or `0b0001_0000` to select directions (P14=0)._


#### 10. High RAM (`0xFF80-0xFFFE`)
- 127 bytes of fast RAM (“zero page”) for frequently accessed variables.

#### 11. Interrupt-Enable (`0xFFFF`)
- Single byte: bits 0–4 enable the five interrupts (V-Blank, LCD-STAT, Timer, Serial, Joypad).
- Write it like normal RAM; on a read you just return the stored byte.
