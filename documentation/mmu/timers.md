# Timers Implementation Documentation

## Overview
The Game Boy timer system provides hardware-based timing functionnality for games. It consists of a constantly incrementing divider and a configurable timer that can trigger interrupts at various frequencies. This is essential for game timing, animations, music, sound effects, and any time-based gameplay mechanics.

## Timer Registers
The timer system uses four hardware registers:

### DIV (0xFF04) - Divider Register
- **Read**: Returns the upper 8 bits of an internal 16-bit counter
- **Write**: Any write resets the entire 16-bit counter to 0 (written value is ignored)
- **Frequency**: Increments at CPU clock rate (4.194304 MHz), upper byte increments at 16384 Hz
- **Use cases**: Random number generation, basic timing, frame pacing

### TIMA (0xFF05) - Timer Counter
- **Read/Write**: 8-bit counter that increments at a configurable frequency
- **Behavior**: When it overflows (0xFF -> 0x00), triggers Timer interrupt and reloads with TMA value
- **Use cases**: Precise timing events, music tempo, synchronized game events

### TMA (0xFF06) - Timer Modulo
- **Read/Write**: 8-bit reload value for TIMA
- **Behavior**: When TIMA overflows, it's set to TMA (not to 0)
- **Use cases**: Control how often timer interrupts occur by setting the starting value

### TAC (0xFF07) - Timer Control
- **Bit 2**: Timer Enable (1 = enabled, 0 = disabled)
- **Bits 0-1**: Clock Select (frequency)
    - `00`: 4096 Hz (CPU clock/1024)
    - `01`: 262144 Hz (CPU clock/16)
    - `10`: 65536 Hz (CPU clock/64)
    - `11`: 16384 Hz (CPU clock/256)
- **Bits 3-7**: Unused

## Frequency Selection Explained
The timer frequencies are derived by watching specific bits of the internal DIV counter:
| TAC bits 0-1 | Frequency | Cycles per TIMA increment | DIV bit watched |
|--------------|-----------|---------------------------|-----------------|
| 00 | 4096 Hz | 1024 | Bit 9 |
| 01 | 262144 Hz | 16 | Bit 3 |
| 10 | 65536 Hz | 64 | Bit 5 |
| 11 | 16384 Hz | 256 | Bit 7 |

**How it works**: TIMA increments when the selected bit of DIV transitions from 1 to 0 (falling
edge detection). This is why the implementation tracks `previous_and_result`.
**Example**: With TAC = 0b101 (enabled, frequency 01):
- DIV bit 3 is watched
- Every 16 CPU cycles, bit 3 toggles from 1 to 0
- On that falling edge, TIMA increments
- Result: TIMA increments at 262144 Hz

## Timer Overflow and Interrupts
When TIMA overflows:
1. TIMA is set to TMA value (not 0)
2. A Timer interrupts is requested (IF bit 2 is set)
3. If interrupts are enabled, CPU jumps to interrupt handler at 0x0050

**Example**:
- TMA = 0xF0, TIMA = 0xFF
- TIMA increments -> 0x00 (overflow!)
- TIMA is loaded with TMA -> TIMA = 0xF0
- Timer interrupt is triggered

This allows precise control over interrupt frequency:
- TMA = 0x00: Interrupt every 256 TIMA increments
- TMA = 0xFF: Interrupt every 1 TIMA increment (very frequent!)
- TMA = 0xF0: Interrupt every 16 TIMA increments

## Current Implementation
### Timers Structure
```rust
#[derive(Default)]
pub struct Timers {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    previous_and_result: bool,
} 
```

**Fields**:
`div: u16`: Internal 16-bit divider counter.
- Increments every CPU cycle (M-cycle, actually)
- Only the upper 8 bits are exposed via the DIV register (0xFF04)
- Lower 8 bits are used for falling edge detection
**Why u16?**: The hardware has a 16-bit internal counter even though the DIV
register only shows 8 bits. The lower bits determine when TIMA increments.

`tima: u8`: Timer counter value (TIMA register).
- Increments at the frequency selected by TAC
- Triggers interrupt on overflow

`tma: u8`: Timer modulo value (TMA register).
- Reload value for TIMA when it overflows

`tac: u8`: Timer control value (TAC register).
- Bit 2: enable/disable
- Bits 0-1: frequency selection

`previous_and_result: bool`: Stores the previous state of the falling edge detector.
- Used to detect when (DIV bit AND timer_enabled) transitions from true to false
- This transition is when TIMA increments

### Address Constants
```rust
const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;
```
Memory addresses for the four timer registers. Used to avoid magic numbers in the code.

### tick() - Core Timer Logic
```rust
pub fn tick(&mut self) -> bool
```
**Purpose**: Advances the timer state by one CPU cycle. Called once per M-cycle by the MMU.
**Returns**: `true` if TIMA overflowed (interrupt should be requested), `false` otherwise.

#### Step-by-step breakdown:
#### Step 1: Increment DIV
```rust
self.div = self.div.wrapping_add(1);
```
- Increments the internal 16-bit DIV counter
- `wrapping_add` ensures 0xFFFF + 1 = 0x0000 without panicking
- This happens every single CPU cycle regardless of timer enable state

#### Step 2: Check Timer Enable
```rust
let enabled = (self.tac & 0b100) > 0;
```
- Extracts bit 2 of TAC (the enable bit)
- `& 0b100` isolates bit 2
- `> 0` converts to bool: true if bit 2 is set, false otherwise

#### Example:
- TAC = 0b00000101 -> enabled = true
- TAC = 0b00000001 -> enabled = false

#### Step 3: Calculate Frequency Mask
```rust
let mask = 0b1
    << match self.tac & 0b11 {
        0b00 => 9,
        0b01 => 3,
        0b10 => 5,
        0b11 => 7,
        _ => unreachable!(),
    };
```
**Purpose**: Creates a mask to select the appropriate bit of DIV based on the frequency.
**How it works**:
1. `self.tac & 0b11` extracts bits 0-1 (frequency select)
2. Match converts to bit position: 9, 3, 5 or 7
3. `0b1 << N` creates a mask with bit N set to 1

#### Frequency to bit mapping:
- `00` -> bit 9 -> mask = 0b0000001000000000 -> falling edge every 1024 cycles
- `01` -> bit 3 -> mask = 0b0000000000001000 -> falling edge every 16 cycles
- `10` -> bit 5 -> mask = 0b0000000000100000 -> falling edge every 64 cycles
- `11` -> bit 7 -> mask = 0b0000000010000000 -> falling edge every 256 cycles

**Why these specific bits?**: Each bit position creates the exact frequency specified in the Game Boy hardware documentation.

**Example**: TAC = 0b00000101 (enabled, frequency 01)
- Frequency bits = 0b01
- Bit position = 3
- Mask = 0b00001000

#### Step 4: Detect Current Bit State
```rust
let kept_bit = (self.div & mask) > 0;
let and_result = kept_bit && enabled;
```
**Purpose**: Determines if the conditions for TIMA increment are currently met.
- `self.div & mask`: Isolates the selected bit from DIV
- `kept_bit`: true if that bit is currently 1
- `and_result`: true if bit is 1 AND timer is enabled

**Example**:
- DIV = 0b0000000000001100, mask = 0b00001000
- `div & mask` = 0b00001000 (non-zero)
- `kept_bit` = true
- If enabled = true -> `and_result` = true

#### Step 5: Falling Edge Detection
```rust
let mut overflowed = false;
if self.previous_and_result && !and_result {
```
**Purpose**: Detects when `and_result` transitions from true to false (falling edge).
**When does this happen?**
- The watched DIV bit transitions from 1 to 0, OR
- Timer is disabled (enabled changes from true to false)

**Why falling edge?**: This matches Game Boy hardware behavior. TIMA increments on the falling edge
of the selected DIV bit when the timer is enabled.

**The condition `previous_and_result && !and_result`**:
- Was true last cycle AND is false this cycle = falling edge detected

#### Step 6: Increment TIMA and Handle Overflow
```rust
let result = self.tima.wrapping_add(1);
if result == 0 {
    self.tima = self.tma;
    overflowed = true
} else {
    self.tima = result;
}
```
**When falling edge detected**:
1. Increment TIMA by 1 (with wrapping)
2. Check if result is 0 (means it wrapped from 0xFF to 0x00)
3. If overflow:
    - Set TIMA to TMA value
    - Set `overflowed` flag to true
4. If no overflow:
    - Store the incremented value in TIMA

**Example overflow scenario**:
- TIMA = 0xFF, TMA = 0x10
- `result = 0xFF.wrapping_add(1)` = 0x00
- `result == 0` -> true
- `self.tima = 0x10`
- `overflowed = true`

**Example no overflow**:
- TIMA = 0x05
- `result = 0x05.wrapping_add(1)` = 0x06
- `result == 0` -> false
- `self.tima = 0x06`

#### Step 7: Update State and Return
```rust
self.previous_and_result = and_result;
overflowed
```
- Saves current `and_result` for next cycle's falling edge detection
- Returns whether an overflow occured (signals interrupt needed)

### write_byte() - Register Writes
```rust
pub fn write_byte(&mut self, addr: u16, value: u8) {
    match addr {
        DIV_ADDR => self.div = 0,
        TIMA_ADDR => self.tima = value,
        TMA_ADDR => self.tma = value,
        TAC_ADDR => self.tac = value,
        _ => unreachable!(),
    }
}
```
#### DIV_ADDR(0xFF04):
- Any write resets DIV to 0, regardless of the value written
- This is hardware behavior: writing to DIV resets the internal 16-bit counter
- Games use this to reset timing or resynchronize events

**Example**:
```rust
timers.write_byte(0xFF04, 0xFF); // Value doesn't matter
// div is now 0x0000
```
**TIMA_ADDR (0xFF05)**: Directly sets TIMA value. Games can modify the timer counter.
**TMA_ADDR (0xFF06)**: Sets the reload value. Changes take effect on next TIMA overflow.
**TAC_ADDR (0xFF07)**: Sets timer control. Changes enable state and frequency immediately.
**unreachable!()**: The MMU routing ensures this function is only called for addresses 0xFF04-0xFF07, so any other address is a programming error.

### read_byte() - Register Reads
```rust
pub fn read_byte(&self, addr: u16) -> u8 {
    let a_box = Box::new(18);
    match addr {
        DIV_ADDR => (self.div >> 8) as u8,
        TIMA_ADDR => self.tima,
        TMA_ADDR => self.tma,
        TAC_ADDR => self.tac,
        _ => unreachable!(),
    }
}
```

#### DIV_ADDR (0xFF04):
```rust
(self.div >> 8) as u8
```
- Returns only the upper 8 bits of the 16-bit internal counter
- `>> 8` shifts right 8 bits
- Cast to u8 discards any remaining upper bits

**Why only upper 8 bits?** The Game Boy hardware only exposes the upper byte of the internal 16-bit DIV counter.
The lower 8 bits are internal and used for the falling edge detection mechanism.

**Example**:
- Internal `div = 0x1A3F`
- `div >> 8` = 0x001A
- Cast to u8 = 0x1A
- Reading DIV register returns 0x1A

**Other registers**: Return their values directly.

## Implementation Status
### Completed
- ✅ DIV counter increments every cycle
- ✅ DIV register read returns upper 8 bits
- ✅ DIV reset on write
- ✅ TIMA increments at configurable frequencies
- ✅ Falling edge detection for TIMA increment
- ✅ All four frequency modes (4096, 262144, 65536, 16384 Hz)
- ✅ Timer enable/disable via TAC bit 2
- ✅ TIMA overflow detection
- ✅ TMA reload on overflow
- ✅ Overflow flag returned to trigger interrupt


last modification: 2026/01/22