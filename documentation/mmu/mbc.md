# MBC (Memory Bank Controller) Implementation

## Overview
The Game Boy can only directly address 32KB of ROM (0x0000-0x7FFF). Larger cartridges use a Memory Bank Controller (MBC) to "switch" between different ROM banks, allowing games to use much more memory than the address space would normally allow.

## Memory Layout
- **0x0000-0x3FFF**: Bank 0 (fixed, always points to the first 16KB of ROM).
- **0x4000-0x7FFF**: Bank N (switchable, N is determined by the MBC).

## MBC Types
- **MBC0**: No banking, maximum 32KB ROM.
- **MBC1**: Up to 2MB ROM + 32KB RAM.
- **MBC2**: Up to 256KB ROM (16 banks) + 512x4 bits built-in RAM (no external RAM support).
- **MBC3**: Like MBC1 + RTC (Real Time Clock) for save timestamps.
- **MBC5**: Up to 8MB ROM + 128KB RAM.

## Current Implementation
### Data Structure
```rust
pub struct Mbc {
    banks: Vec<[u8; 0x4000]>, // Vector of 16KB ROM banks
    current: usize, // Currently selected switchable bank
}
```

### Fields:
- `banks`: A vector where each element is a 16KB (0x4000 bytes) array representing one ROM bank.
- `current`: The index of the ROM bank currently mapped to address space 0x4000-0x7FFF (default to 1).

### Initialization(`new`):
The constructor takes a raw ROM image and splits it into 16KB banks:
1. **Bank Creation Loop**:
- Iterates through the ROM data in 16KB chunks
- For each chunk, creates a new `[u8; 0x4000]` array.
- Copies ROM data into the array (handles partial banks at the end).
- Adds the bank to the `banks` vector.

2. **Minimum Banks Guarantee**:
- If the ROM is smaller than 32KB (less  than 2 banks), pads with empty banks.
- This ensures we always have at least bank 0 (fixed) and bank 1 (switchable).

3. **Initial State**:
- Sets `current = 1`, meaning bank 1 is initially mapped to 0x4000-0x7FFF.

**Example**: A 20KB ROM would create:
- Bank 0: First 16KB of ROM data.
- Bank 1: Remaining 4KB of ROM data + 12 KB of zeros.

### Reading (`read`):
```rust
pub fn read(&self, addr: u16) -> u8
```

Handles all reads from the ROM address space (0x0000-0x7FFF):
- **If `addr < 0x4000`** (Bank 0 region):
    - Returns `banks[0][addr]`.
    - This regions is fixed and always points to the first ROM bank.
- **If `addr >= 0x4000`** (Switchable bank region):
    - Calculate offset: `off = addr - 0x4000`.
    - Returns `banks[current][off]`.
    - This allows switching which 16KB bank is visible in this region.
    
**Example**: Reading from address 0x5000 with `current = 3`:
- Offset = 0x5000 - 0x4000 = 0x1000
- Returns byte at position 0x1000 in bank 3.

### Writing (`write`):
```rust
pub fn write(&mut self, addr: u16, val: u8)
```

**Currently empty** - this is intentional placeholder code.

In the future, MBC implementation, writes to ROM addresses don't actually modify ROM (which is read-only). Instead, they control the MBC's behavior:
- **0x0000-0x1FFF**: Enable/disable external RAM (0x0A = enable)
- **0x2000-0x3FFF**: Select ROM bank number (lower bits)
- **0x4000-0x5FFF**: Select RAM bank OR upper ROM bank bits
- **0x6000-0x7FFF**: Banking mode selection

## Implementation Status
- ✅ ROM bank storage and organization
- ✅ Reading from fixed bank 0
- ✅ Reading from switchable banks
- ✅ Minimum 2-bank guarantee
- ❌ Bank switching control (write implementation)
- ❌ External RAM support
- ❌ MBC type detection from cartridge header (byte at 0x0147)
- ❌ RAM enable/disable
- ❌ Different MBC variants (MBC1/3/5 specific logic)


last modification: 2026/01/22