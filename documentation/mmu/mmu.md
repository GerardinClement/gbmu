# MMU (Memory Management Unit) Implementation
## Overview
The MMU is the central memory router for the Game Boy emulator. It intercepts all memory reads and writes from the CPU and routes them to the appropriate hardware component or memory region. Think of it as a switchboard operator directing calls to the right department.

## Why the MMU Exists
The Game Boy has a 64KB address space (0x0000-0xFFFF), but this space is divided among many different hardware components:
- ROM cartidge (via MBC)
- Video RAM (managed by PPU)
- Work RAM
- I/O registers (joypad, sound, LCD control, timers, etc.)
- High RAM (HRAM)
- Object Attribute Memory (OAM - sprite data)
The MMU's job is to:
1. Determine which component "owns" a given memory address
2. Route reads/writes to that component
3. Handle special cases (mirrors, unusable regions, read-only areas)

## Memory Map
```
0x0000-0x3FFF: ROM Bank 0 (fixed)
0x4000-0x7FFF: ROM Bank N (switchable via MBC)
0x8000-0x9FFF: VRAM (Video RAM)
0xA000-0xBFFF: External RAM (cartridge RAM, if present)
0xC000-0xDFFF: Work RAM (WRAM)
0xE000-0xFDFF: Echo RAM (mirror of 0xC000-0xDDFF)
0xFE00-0xFE9F: OAM (Object Attribute Memory - sprites)
0xFEA0-0xFEFF: Unusable (reads return 0xFF, writes ignored)
0xFF00-0xFF7F: I/O Registers
0xFF04-0xFF07: Timer registers (special handling)
0xFF0F: Interrupt Flag (IF)
0xFF80-0xFFFE: High RAM (HRAM)
0xFFFF: Interrupt Enable (IE)
```

## Current Implementation
### MemoryRegion Enum
```rust
#[derive(PartialEq, Eq, Debug)]
pub enum MemoryRegion {
    Mbc,
    Vram,
    ERam,
    Wram,
    Mran,
    Oam,
    Unusable,
    InterruptFlag,
    Timers,
    Io,
    HRam,
    InterruptEnable,
}
```
**Purpose**: Type-safe representation of memory regions. Each variant corresponds to a range of addresses.
**Design choice**: `InterruptFlag`, `Timers`, and `InterruptEnable` are technically part of the I/O region, but are separated for special handling. This makes routing logic clearer and allows direct delegation to specific components.
**Derivations**:
- `PartialEq, Eq`: Allows comparing regions (used in tests)
- `Debug`: Enables printing for debugging

### MemoryRegion::from()
```rust
pub fn from(addr: u16) -> Self {
    match addr {
        0x0000..=0x7FFF => MemoryRegion::Mbc,
        0x8000..=0x9FFF => MemoryRegion::Vram,
        0xA000..=0xBFFF => MemoryRegion::ERam,
        0xC000..=0xDFFF => MemoryRegion::Wram,
        0xE000..=0xFDFF => MemoryRegion::Mram,
        0xFE00..=0xFE9F => MemoryRegion::Oam,
        0xFEA0..=0xFEFF => MemoryRegion::Unusable,
        0xFF04..=0xFF07 => MemoryRegion::Timers,
        0xFF0F => MemoryRegion::InterruptFlag,
        0xFF00..=0xFF7F => MemoryRegion::Io,
        0xFF80..=0xFFFE => MemoryRegion::HRam,
        0xFFFF => MemoryRegion::InterruptEnable,
    }
}
```
**Purpose**: Converts a 16-bit address to its corresponding memory region.
**How it works**: Uses Rust's pattern matching with inclusive ranges (`..=`). The patterns are evaluated top-to-bottom, so order matters.
**Critical detail - pattern order**: Specific patterns must come before general ones:
- `Timers` (0xFF04-0xFF07) appears before `Io` (0xFF00-0xFF7F)
- `InterruptFlag` (0xFF0F) appears before `Io`

If `Io` came first, addresses like 0xFF0F would always match `Io` and never reach `Interrupt Flag`.
**Example**:
- Address 0x8123 -> matches `0x8000..=0x9FFF` -> returns `MemoryRegion::Vram/`
- Address 0xFF06 -> matches `0xFF04..=0xFF07` -> returns `MemoryRegion::Timers` (not `Io`).

### MemoryRegion::to_address()
```rust
pub fn to_address(&self) -> u16 {
    match self {
        memoryRegion::Mbc => 0x0000,
        MemoryRegion::Vram => 0x8000,
        // etc.
    }
}
```
**Purpose**: Returns the starting address of a memory region.
**Usage**: Primarily used to avoid magic numbers in code. For example, in `tick_timers()`, instead of writing `0xFF0F`, we write `MemoryRegion::InterruptFlag.to_address()`, which is more self-documenting.

### `Mmu` Structure
```rust
pub struct Mmu {
    data: [u8; 0x10000],
    cart: Mbc,
    interrupts: InterruptController,
    timers: Timers,
}
```
**Fields**:
`data: [u8; 0x10000]`: A 64KB array representing the entire addressable memory space.
- Contains VRAM, WRAM, OAM, I/O registers, HRAM
- Simple approach: one contiguous array for all regions
- Trade-off: Uses full 64KB even though some regions are special (like ROM which is in `cart`)
- Future consideration, Some regions (like VRAM, OAM) might be better managed by the PPU component.

`cart: Mbc`: The Memory Bank Controller managing the ROM cartridge.
- Handles reads from 0x0000-0x7FFF (ROM area)
- Handles writes to ROM area (which control bank switching, not actual ROM writes)
- Manages external cartridge RAM if present

`interrupts: InterruptController`: The interrupt management system.
- Manages IE (0xFFFF) and IF (0xFF0F) registers
- Determines which interrupts are pending and enabled

`timers: Timers`: The timer hardware registers.
- DIV (0xFF04): Divider register
- TIMA (0xFF05): Timer counter
- TMA (0xFF06): Timer modulo
- TAC (0xFF07): Timer control

### Construction
```rust
pub fn new(rom_image: &[u8]) -> Self {
    Mmu {
        data: [0; 0x10000],
        cart: Mbc::new(rom_image),
        interrupts: InterruptController::new(),
        timers: Timers::default(),
    }
}
```
### Initialization:
1. Allocates 64KB of zero-initialized memory
2. Loads the ROM into the MBC
3. Creates interrupt controller with IE=0, IF=0
4. Creates timers with default state

### Timer Tick
```rust
pub fn tick_timers(&mut self) {
    if self.timers.tick() {
        let interrupt_flags_addr = MemoryRegion::InterruptFlag.to_address();
        let mut interrupt_flags = self.read_byte(interrupt_flag_addr);
        interrupt_flags |= 0b100;
        self.write_byte(interrupt_flags_addr, interrupt_flags);
    }
}
```
**Purpose**: Advances the timer hardware by one cycle and handles timer overflow.
**How it works**:
1. Calls `self.timers.tick()` to advance the timer state
2. If `tick()` returns `true`, the timer has overflowed (TIMA wrapped around)
3. Gets the IF register address (0xFF0F) using `to_address()`
4. Reads current interrupt flags
5. Sets bit 2 (Timer interrupt) using OR: |= 0b100
6. Writes the updated flags back

### read_byte() - Memory Read Router
```rust
pub fn read_byte(&self, addr: u16) -> u8 {
    match MemoryRegion::from(addr) {
```
**Purpose**: Routes memory reads to the appropriate component of storage.
#### Pattern 1 - MBC (ROM):
```rust
MemoryRegion::Mbc => self.cart.read(addr),
```
Delegates to the MBC, which handles bank switching logic internally.

#### Pattern 2 - Echo RAM (MRAM):
```rust
MemoryRegion::Mram => {
    let mirror = addr - 0x2000;
    self.data[mirror as usize]
}
```
Echo RAM (0xE000-0xFDFF) mirrors WRAM (0xC000-0xDDFF).
- Example: Reading 0xE123 -> mirror = 0xE123-0x2000 = 0xC123
- Returns `data[0xC123]`, which is in the WRAM region
- This implements the hardware mirroring behavior

#### Pattern 3 - Unusable Region:
```rust
MemoryRegion::Unusable => 0xFF,
```
Region 0xFEA0-0xFEFF always reads as 0xFF (all bits set). This matches actual Game Boy hardware behavior.

#### Pattern 4 - Interrupt Registers:
```rust
MemoryRegion::InterruptFlag => self.interrupts.read_interrupt_flag(),
MemoryRegion::InterruptEnable => self.interrupts.read_interrupt_enable(),
```
Delegates to the interrupt controller, which handles proper bit masking:
- IF read: returns lower 5 bits with upper 3 bits forced to 1
- IE read: returs lower 5 bits with upper 3 bits forced to 0

#### Pattern 5 - Default (Direct Memory Access):
```rust
_ => self.data[addr as usize],
```
For all other regions (VRAM, WRAM, OAM, I/O, HRAM), reads directly from the `data` array at the specified address.
Regions handled by this default case (for now):
- VRAM (0x8000-0x9FFF)
- External RAM (0xA000-0xBFFF) - currently just reads from data array
- WRAM (0xC000-0xDFFF)
- OAM (0xFE00-0xFE9F)
- Most I/O registers (0xFF00-0xFF7F except timers and IF)
- HRAM (0xFF80-0xFFFE)

### write_byte() - Memory Write Router
```rust
pub fn write_byte(&mut self, addr: u16, val: u8) {
    match MemoryRegion::from(addr) {
```
**Purpose**: Routes memory writes to the appropriate component or storage.

#### Pattern 1 - MBC (ROM area):
```rust
MemoryRegion::Mbc => self.cart.write(addr, val),
```
Writes to ROM addresses (0x0000-0x7FFF) don't modify ROM. Instead, they're interpreted as control commands by the MBC:
- 0x0000-0x1FFF: Enable/disable external RAM.
- 0x2000-0x3FFF: Select ROM bank.
- 0x4000-0x5FFF: Select RAM bank or upper ROM bits.
- 0x6000-0x7FFF: Banking mode selection.

#### Pattern 2 - Echo RAM (MRAM):
```rust
MemoryRegion::Mram => {
    let mirror = addr - 0x2000;

    self.data[mirror as usize]
}
```
Writing to Echo RAM (0xE000-0xFDFF) actually writes to WRAM (0xC000-0xDDFF).
- Example: Writing to 0xE050 - 0x2000 = 0xC050.
- Actually writes to `data[0xC050]`.
- Subsequent reads from either 0xE050 or 0xC050 will returns this value.

#### Pattern 3 - Timers:
```rust
MemoryRegion::Timers => {
    self.timers.write_byte(addr, val);
}
```
Delegates to the Timers component for handling writes to:
- DIV (0xFF04): Writing any value resets to 0
- TIMA (0xFF05): Timer counter value
- TMA (0xFF06): Timer modulo (reload value)
- TAC (0xFF07): Timer control (enable, speed)

#### Pattern 4 - Unusable Region:
```rust
MemoryRegion::Unusable => {}
```
Writes to 0xFEA0-0xFEFF are silently ignored. Empty block = no operation. Hardware accurate behavior.

#### Pattern 5 - Interrupt Registers:
```rust
MemoryRegion::InterruptFlag => self.interrupts.write_interrupt_flag(val),
MemoryRegion::InterruptEnable => self.interrupts.write_interrupt_enable(val),
```
Delegates to interrupt controller with proper masking (only lower 5 bits are stored).

#### Pattern 6 - Default (Direct Memory Access):
```rust
_ => self.data[addr as usize] = val,
```
For all other regions, writes directly to the `data` array.

### Interrupt Helper Methods
```rust
pub fn read_interrupt_enable(&self) -> u8 {
    self.interrupts.read_interrupt_enable()
}

pub fn read_interrupt_flag(&self) -> u8 {
    self.interrupts.read_interrupt_flag()
}

pub fn interrupts_next_request(&self) -> Option<Interrupt> {
    self.interrupts.next_request()
}

pub fn interrupts_clear_request(&mut self, interrupt: Interrupt) {
    self.interrupts.clear_request(interrupt);
}

pub fn interrupts_request(&mut self, interrupt: Interrupt) {
    self.interrupts.request(interrupt);
}
```
**Purpose**: Provide a clean API for interrupt management without exposing internal MMU structure.
#### Usage by CPU:
- After each instruction: call `interrupts_next_request()` to check for pending interrupts
- When servicing interrupt: call `interrupts_clear_request(interrupt)`
- When hardware event occurs: call `interrupts_request(interrupt)`

## Implementation Status
### Completed
- ✅ Memory region identification and routing
- ✅ ROM/cartridge access via MBC
- ✅ Echo RAM mirroring (0xE000-0xFDFF <-> 0xC000-0xDDFF)
- ✅ Unusable region behavior (reads 0xFF, writes ignored)
- ✅ Interrupt register access (IE, IF)
- ✅ Timer integration
- ✅ Direct memory access for standard regions
- ✅ Interrupt management API

### Missing/TODO
- ❌ External RAM (0xA000-0xBFFF) - currently just uses `data` array
- ❌ PPU integration - VRAM and OAM should eventually be managed by PPU
- ❌ I/O register implementations - most I/O registers need special behavior
- ❌ DMA (Direct Memory Acces) for OAM transfers
- ❌ Memory access timing (some regions are slower than others)
- ❌ PPU memory access restrictions (VRAM/OAM inaccessible during certain PPU modes)
- ❌ Joypad register (0xFF00)
- ❌ Sound registers (0xFF10-0xFF26)
- ❌ Serial transfer registers (0xFF01-0xFF02)