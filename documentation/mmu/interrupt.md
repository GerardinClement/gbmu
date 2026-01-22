# Interrupt System Implementation
## Overview
Interrupts allow hardware components to signal the CPU that an important event has occured. Instead of constantly polling for events (wasting CPU cycles), the hardware can "interrupt" the CPU's normal execution to handle time-sensitive tasks.

## The Game Boy's 5 Interrupt Types
Listed in priority order (highest to lowest):
1. **V-Blank (bit 0)**: Triggered when the PPU finishes drawing frame (scanline 144). This is the safe window to modify VRAM without visual artifacts.
2. **LCD STAT (bit 1)**: Triggered by various PPU conditions (mode changes, scanline coincidence).
3. **Timer (bit 2)**: Triggered when the internal timer overflows.
4. **Serial (bit 3)**: Triggered when a serial transfer completes (link cable communication).
5. **Joypad (bit 4)**: Triggered when a button is pressed.

## Hardware Registers
The interrupt system uses two mains registers:
- **IE (Interrupt Enable, 0xFFFF)**: Each bit enables/disables a specific interrupt type.
- **IF (Interruption Flag, 0xFF0F)**: Each bit indicates whether an interrupt has been requested.

**Bit layout** (both registers use the same pattern):
```
Bit 4: Joypad
Bit 3: Serial
Bit 2: Timer
Bit 1: LCD STAT
Bit 0: V-Blank
Bits 5-7: Unused (always 1 when reading IF, always 0 when reading IE)
```

## Interrupt Execution Flow
1. **Request**: A hardware component sets a bit in IF (e.g., PPU sets bit 0 for V-Blank)
2. **Check**: CPU checks if IME (Interrupt Master Enable) is on AND the corresponding bit in IE is set.
3. **Handle**: If both conditions are met, the CPU:
    - Pushes current PC onto the stack.
    - Clears IME (disables further interrupts).
    - Clears the IF bit.
    - Jumps to the interrupt's vector address.
4. **Execute**: Game code handles the interrupt.
5. **Return**: RETI instruction restores PC and  re-enables IME.

## Vector Addresses
When an interrupt is serviced, the CPU jumps to these fixed addresses:
- V-Blank: 0x0040
- LCD STAT: 0x0048
- Timer: 0x0050
- Serial: 0x0058
- Joypad: 0x0060

## Current Implementation

### Interrupt Enum

```rust
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interrupt {
    VBlank = 0b00000001,
    LcdStat = 0b0000010,
    Timer = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}
```
**Purpose**: Type-safe representation of the five interrupt types.

### Design Choices:
- `#[repr(u8)]`: Forces Rust to represent this enum as a u8 in memory, allowing direct casting to bit values.
- Each variant is a power of 2 (single bit set), matching the bit position in IE/IF registers.
- `Copy + Clone`: Allows easy copying of interrupt values.
- `Debug + PartialEq`: Enables printing and comparision in tests.

**Usage**: `Interrupt::VBlank as u8` yields `0b00000001`, which can be directly used with bitwise operations on IE/IF.

### Vector Address Method
```rust
pub fn vector(self) -> u16 {
    match self {
        Interrupt::VBlank => 0x40,
        Interrupt::LcdStat => 0x48,
        Interrupt::Timer => 0x50,
        Interrupt::Serial => 0x58,
        Interrupt::Joypad => 0x60,
    }
}
```
Returns the memory address where the CPU should jump when servicing this interrupt. These addresses are defined by Game Boy hardware specification.

### InterruptController Structure
```rust
pub struct InterruptController {
    ienable: u8, // IE register (0xFFFF)
    iflag: u8, // IF register (0xFF0F)
}
```
#### Fields:
- `ienable`: Represents the IE (Interrupt Enable) register. Each bit enables one interrupt type.
- `iflag`: Represents the IF (Interrupt Flag) register. Each bit indicates a pending interrupt request.

Both are `u8` values where only the lower 5 bits are meaningful.

### IE Register Access
```rust
pub fn read_interrupt_enable(&self) -> u8 {
    self.ienable & 0b00011111
}

pub fn write_interrupt_enable(&mut seulf, val: u8) {
    self.ienable = val & 0b00011111;
}
```
**Read**: Returns `ienable` with upper 3 bits masked to 0 (hardware behavior).
**Write**: Stores only the lower 5 bits of the value. Upper 3 bits are always ignored.
**The mask `0b00011111` (0x1F)**: Ensures only bits 0-4 are used, matching actual hardware behavior where only 5 interrupt types exist.

### IF Register Access
```rust
pub fn read_interrupt_flag(&self) -> u8 {
    self.iflag | 0b11100000
}
 pub fn write_interrupt_flag(&mut self, val: u8) {
    self.iflag = val & 0b00011111;
 }
 ```
 **Read**: Returns `iflag` with upper 3 bits forced to 1. This matches actual Game Boy hardware. When reading IF, bits 5-7 are always 1.
 **Write**: Stores only the lower 5 bits. Games can write to IF to clear interrupt requests.

### Requesting and Clearing Interrupts
```rust
pub fn request(&mut self, interrupt: Interrupt) {
    self.iflag |= interrupt as u8;
}
```
**Purpose**: Hardware components call this to request an interrupt.
**How it works**:
- Converts the enum to its u8 value (e.g., `Timer` -> `0b00000100`)
- Uses bitwise OR to set the corresponding bit to 1.
- Multiple interrupts can be pending simultaneously.
**Example**: If `iflag = 0b00000001` (V-Blank pending) and we reqest Timer:
- `iflag |= 0b00000100`
- Result: `iflag = 0b0000101` (both V-Blank and Timer pending).
```rust
pub fn clear_request(&mut self, interrupt: Interrupt) {
    self.iflag &= !(interrupt as u8);
}
```
**Purpose**: CPU calls this after servicing an interrupt to clear the request.
**How it works:**
- Converts enum to u8 and applies bitwise NOT (e.g., `!(0b00000100)` -> `0b11111011`)
- Uses bitwise AND to clear only that specific bit
- Other pending interrupts remain unaffected

**Example**: If `iflag = 0b00000101` (V-Blank and Timer) and we clear Timer:
- `iflag &= !(0b00000100)` = `iflag &= 0b11111011`
- Result: `iflag = 0b00000001` (only V-Blank remains)

### Determining Next Interrupt
```rust
pub fn next_request(&self) -> Option<Interrupt> {
    let pending_request = self.ienable & self.iflag;
}
```
#### Step 1 - Calculate pending requests:
- Uses bitwise AND between IE and IF
- Result: bits that are 1 in BOTH registers (requested AND enabled)
- Only these interrupts should actually be serviced

**Example**:
- `ienable = 0b00010101` (V-Blank, Timer, Joypad enabled)
- `iflag = 0b00001110` (LCD STAT, Timer, Serial requested)
- `pending_request = 0b00000100` (only Timer is both enabled and requested)
```rust
[
    Interrupt::VBlank,
    Interrupt::LcdStat,
    Interrupt::Timer,
    Interrupt::Serial,
    Interrupt::Joypad,
]
.iter()
.find(|&&interrupt| pending_request & (interrupt as u8) != 0)
.copied()
```

#### Step 2 - Find highest priority interrupt:
- Creates an array of interrupts **in priority order** (V-Blank highest, Joypad lowest)
- `.iter()`: Creates an iterator over the array
- `.find()`: Returns the first interrupt where the condition is true
- Condition: `pending_request & (interrupt as u8) != 0` checks if this interrupt's bit is set
- `.copied()`: Converts from `Option<&Interrupt>` to `Option<Interrupt>`

**Priority mechanisme**: Since we iterate in priority order and use `.find()` (which stops at the first match), we automatically get the highest priority pending interrupt.

**Return value**:
- `Some(Interrupt::X)` if an enabled interrupt is pending.
- `None` if no enabled interrupts are pending.

**Example with multiple pending**:
- `pending_request = 0b00010101` (V-Blank, Timer, Joypad all pending)
- Iterator checks V-Blank first: `0b00010101 & 0b00000001 != 0` ✅
- Returns `Some(Interrupt::VBlank)` immediatly without checking Timer or Joypad

## Implementation Status
- ✅ All 5 interrupt types defined
- ✅ Vector addresses for all interrupts
- ✅ IE register read/write with correct masking
- ✅ IF register read/write with hardware-accurate upper bits
- ✅ Request and clear individual interrupts
- ✅ Priority-based interrupt selection
- ✅ Proper handling of enabled vs requested interrupts

## Typical usage flow:
1. Hardware component calls `controller.request(Interrupt::VBlank)`
2. CPU checks `controller.next_request()` after each instruction
3. If Some(interrupt) is returned and IME is enabled, CPU:
    - Calls `controller.clear_request(interrupt)`
    - Pushes PC to stack
    - Sets PC to `interrupt.vector()`
    - Disables IME
4. Game code handles interrupt, ends with RETI
5. RETI restores PC and re-enables IME


last modification: 2026/01/22