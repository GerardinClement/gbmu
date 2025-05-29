
## Game Boy Memory Map (0x0000–0xFFFF)


|Range           |	Size     |               Name            |	Purpose / Notes                                     |
|----------------|-----------|-------------------------------|------------------------------------------------------|
|0x0000–0x3FFF	 |   16 KiB	 |   ROM Bank 0	                 |  Fixed startup code, reset/jump vectors, cartridge header (title, MBC type, ROM/RAM size, checksum). Read-only.                                                        |
|0x4000–0x7FFF	 |   16 KiB	 |   ROM Bank 1–N                |	Switchable game code/data via MBC (bank-switching). |
|0x8000–0x9FFF	 |   8 KiB	 |   Video RAM (VRAM)            |	Tile pixel data (0x8000–0x8FFF), tile maps (0x9800–0x9BFF & 0x9C00–0x9FFF).                    |
|0xA000–0xBFFF	 |   8 KiB	 |   External RAM	             |  Cartridge RAM (battery-backed save data). May be banked by MBC.                               |
|0xC000–0xCFFF	 |   4 KiB	 |   Work RAM Bank 0 (WRAM0)     |	Main CPU RAM.                                       |
|0xD000–0xDFFF	 |   4 KiB	 |   Work RAM Bank 1–7 (WRAM1–7) |	Additional banks on CGB (color mode); on DMG always bank 1.                                      |
|0xE000–0xFDFF	 |   7.5 KiB |	Echo RAM                     |	Mirror of 0xC000–0xDDFF. Reads/writes act exactly like WRAM.                                  |
|0xFE00–0xFE9F	 |   160 B	 |   OAM (Sprite Attribute Table)|	40 sprites × 4 B each: Y, X, tile#, flags.          |
|0xFEA0–0xFEFF	 |   96 B	 |   Not Usable                  |	Forbidden. Returns open-bus or $FF.                 |
|0xFF00–0xFF7F	 |   128 B	 |   I/O Registers               |	Joypad, serial, timer/divider, interrupts, audio, LCD control, DMA, CGB extras. Reads/writes dispatch to hardware modules.          |
|0xFF80–0xFFFE	 |   127 B	 |   High RAM (HRAM)             |	Fast “zero-page” RAM for critical data.             |
|0xFFFF–0xFFFF	 |   1 B	 |   Interrupt-Enable (IE)       |	Bitmask enabling V-Blank, LCD STAT, Timer, Serial, Joypad interrupts.                          |

## Region-by-Region Breakdown

#### 1. ROM Bank 0 (`0x0000-0x3FFF`)
- CPU's first instructions and jump vectors (addresses `0x00`, `0x08`, ... `0x38` via `RST`).
- **Cartdridge header** at `0x0100-0x014F`: game title, CGB flag, MBC type, ROM size, RAM size, destination code, mask ROM version, header checksum.
- **Writes** here do **not** overwrite code — instead they become MBC commands once you implement MBC logic.

#### 2. Switchable ROM (`0x4000-0x7FFF`)
- On a write to (`0x2000-0x3FFF`), your MBC will pick which 16 KiB bank appears here.
- **Implementation**: keep `Vec<Vec<u8>>` of 16 KiB pages; on bank-select, `copy_from_slice()` into `data[0x4000..0x8000].

#### 3. VRAM (`0x8000-0x9FFF`)
- **Tile data** (`0x8000-0x8FFF`): each 16 B -> one 8x8 tile (2 bpp). 384 tiles per bank.
- Tile maps (`0x9800-0x9BFF` & `0x9C00-0x9FFF`): 32x32 entries = 1024 B each; tell the PPU which tile to draw at each background cell.
- **On CGB**, a second bank lives in the same addresses when `VBK=1`.

#### 4. External RAM (`0xA000-0xBFFF`)
- Cartridge-provided RAM, usually battery-backed for save games.
- May be banked by the MBC(writes to bank-select registers in `0x0000-0x7FFF`).
- If no RAM present, reads return `$FF` and writes are ignored.

#### 5. Work RAM (`0xC000-0xDFFF`)
- **Bank 0** at `0xC000-0xCFFF`.
- **Bank 1-7** at `0xD000-0xDFFF` on CGB (select via `0xFF70`). On DMG this is always bank 1.
- Used for stacks, variables, local buffers, etc.

#### 6. Echo RAM (`0xE000-0xFDFF`)
- Simple mirror of `0xC000–0xDDFF`.
- **Implementation**: in your read/write, if `0xE000 ≤ addr ≤ 0xFDFF`, do `read_byte(addr-0x2000)` (and same for write).

#### 7. OAM (`0xFE00-0xFE9F`)
- **Sprite Attribute Table**: 40 sprites × 4 bytes:
    1. Y-position
    2. X-position
    3. Tile index
    4. Flags (palette, flip, priority)

- **DMA**: a write to `0xFF46` starts a quick 160 B copy from `val<<8` → `0xFE00–0xFE9F`.

#### 8. Not Usable (`0xFEA0-0xFEFF`)
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


#### 10. High RAM (`0xFF80-0xFFFE`)
- 127 bytes of fast RAM (“zero page”) for frequently accessed variables.

#### 11. Interrupt-Enable (`0xFFFF`)
- Single byte: bits 0–4 enable the five interrupts (V-Blank, LCD-STAT, Timer, Serial, Joypad).
- Write it like normal RAM; on a read you just return the stored byte.