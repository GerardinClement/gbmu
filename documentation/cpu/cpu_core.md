# CPU Core Documentation
## Overview
The CPU (Central Processing Unit) is the brain of the Game Boy emulator. It fetches instructions from memory, decodes them and executes them. The Game Boy uses a Sharp LR35902 processor, which is an 8-bit CPU hybrid between the Zilog Z80 and Intel 8080.

## CPU Architecture
### Sharp LR35902 Specifications
- **8-bit processor** with 16-bit address bus (64KB addressable memory)
- **Clock speed**: ~4.194 MHz (4,194,304 Hz)
- **Instruction set**: Similar to Z80 but with differences (no shadow registers, different timing)
- **Registers**: 8 general-purpose 8-bit registers that can be paired into 4 16-bit registers
- **Stack**: Grows downward in memory, managed by Stack Pointer (SP)

### Memory-Mapped Components
The CPU doesn't work alone. It communicates with:
- **MMU**: Routes all memory accesses
- **PPU**: Graphics rendering (VRAM, OAM)
- **APU**: Sound generation
- **Timers**: Hardware timing
- **Joypad**: Input handling

## Current Implementation
### Cpu Structure
```rust
pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub bus: Arc<RwLock<Mmu>>,
    pub ime: bool,
    pub ime_delay: bool,
    pub halted: bool,
    pub halt_bug: bool,
    tick_to_wait: u8,
}
```
#### Fields:
`registers: Registers`: Contains the 8 general-purpose registers (A, B, C, D, E, H, L) and the flags regsiter (F).
- Can be accessed as 8-bit (A, B, C, D, E, H, L) or 16-bit pairs (AF, BC, DE, HL)
- Also contains Stack Pointer (SP)

`pc: u16`: Program Counter - points to the address of the next instruction to execute.
- Starts at 0x0100 after boot ROM
- Incremented automatically after fetching each instruction
- Modified by jumps, calls, returns, and interrupts

`bus: Arc<RwLock<Mmu>>`: Shared reference to the Memory Management Unit.
- `Arc`: Allows multiple owners (CPU, potentially PPU, etc.)
- `RwLock`: Provides thread-safe read/write access
- All memory reads/writes go through the bus

`ime: bool`: Interrupt Master Enable - global interrupt enable flag.
- `true`: CPU can be interrupted when interrupts are pending
- `false`: CPU ignores interrupts (except HALT behavior)
- Controller by EI (enable) and DI (disable) instructions
- Automatically disabled when servicing an interrupt

`ime_delay: bool`: One-instruction delay for EI instruction.
- EI instruction has a hardware delay: IME is enabled AFTER the next instruction executes
- When EI executes: sets `ime_delay = true`
- After next instruction: `ime` is set to true
- Purpose: Allows patterns like `EI; RETI` to work safely

`halted: bool`: CPU is in HALT mode (low-power state).
- Set by HALT instruction (0x76)
- CPU stops executing instructions but timers/interrupts continue
- Wakes up when an interrupt is pending (IF & IE != 0)
- If IME is disabled during wake-up, triggers halt_bug

`halt_bug: bool`: Implements Game Boy HALT bug hardware quirk.
- Occurs when: HALT executed with IME = 0 and interrupt pending
- Effect: Next byte after HALT is executed twice
- Hardware bug, not an emulation error - real Game Boy does this

`tick_to_wait: u8`: Remaining CPU cycles before next instruction can execute.
- Each instruction takes multiple cycles (4, 8, 16, 20, or 24)
- `tick()` is called every cycle
- When `tick_to_wait` reaches 0, next instruction executes

### Default Initialization
```rust
impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            registers: Registers::default(),
            bus: Arc::new(RwLock::new(Mmu::default())),
            pc: 0x0100,
            ime: false,
            ime_delay: false,
            halted: false,
            halt_bug: false,
            tick_to_wait: 0,
        }
    }
}
```
#### Initial state:
- PC starts at 0x0100 (where cartridge code begins after boot ROM)
- IME disabled (no interrupts initially)
- Not halted
- No pending cycles

### Construction
```rust
pub fn new(bus: Arc<RwLock<Mmu>>) -> Self {
    Cpu {
        pc: 0x0100,
        bus,
        ..Default::default()
    }
}
```
Allows creating a CPU with a specific MMU bus while using default values for everything else.

### Execution Cycle
#### tick() - Per-Cycle Execution
```rust
pub fn tick(&mut self) {
    if self.tick_to_wait > 0 {
        self.tick_to_wait -= 1;
    } else {
        self.tick_to_wait = self.step();
    }
}
```
**Purpose**: Called every CPU cycle (M-cycle) by the emulator main loop.
**Behavior**:
1. If waiting for an instruction to complete (`tick_to_wait > 0`):
    - Decrement the counter
    - Do nothing else (instruction still executing)
2. If no cycles remaining (`tick_to_wait == 0`):
    - Execute next instruction via `step()`
    - Store how many cycles that instruction takes

**Example execution flow**:
```
Cycle 1: tick() -> tick_to_wait=0 -> step() executes ADD (4 cycles) -> tick_to_wait=4
Cycle 2: tick() -> tick_to_wait=3 (decrements, does nothing else)
Cycle 3: tick() -> tick_to_wait=2
Cycle 4: tick() -> tick_to_wait=1
Cycle 5: tick() -> tick_to_wait=0 -> step() executes next instruction
```
This accurately models timings: multi-cycle instructions span multiple emulator cycles.

#### step() - Instruction Execution
```rust
pub fn step(&mut self) -> u8 {
    if self.handle_halt_state() == StepStatus:: Halted {
        return 4;
    }
    if self.handle_ime_state() == StepStatus::Halted {
        return 5;
    }

    let instruction_byte = self.bus.read().unwrap().read_byte(self.pc);
    let tick_to_wait = self.execute_instruction(instruction_byte);

    self.handle_halt_bug();
    self.handle_ime_delay();

    tick_to_wait
}
```
**Purpose**: Executes one complete "step". Either waiting during HALT, servicing an interrupt, or executing an instruction.

**Execution order** (critical, order matters):
1. **Check HALT state**: If halted and no interrupt pending, stay halted (return 4 cycles)
2. **Check for interrupts**: If IME enabled and interrupt pending, service it (return 5 cycles)
3. **Fetch instruction**: Read byte at PC from memory
4. **Execute instruction**: Decode and execute, get cycle count
5. **Handle hatl bug**: If halt_bug flag set, decrement PC
6. **Handle IME delay**: If ime_delay set, activate IME
7. **Return cycle count**: Tell tick() how many cycles to wait

**Return values**:
- 4: HALT state (arbitrary timing while waiting)
- 5: Interrupt serviced (fixed timing for interrupt handling)
- Variable: Instruction-specific cycle count (4-24 cycles)

### HALT Management
#### handle_halt_state()
```rust
fn handle_halt_state(&mut self) -> StepStatus {
    if self.halted {
        let bus = self.bus.read().unwrap();
        let iflag = bus.read_interrupt_flag();
        let ienable = bus.read_interrupt_enable();

        if ienable & iflag == 0 {
            return StepStatus::Halted;
        }

        self.halted = false;

        if !self.ime {
            self.halt_bug = true;
        }
    }
    StepStatus::Continue
}
```
**What is HALT?** HALT (instruction 0x76) puts the CPU into low-power mode. It stops executing instructions to save energy but hardware (timers, PPU) continues running.

**How it works**:
**Case 1 - Stay halted**: If `halted` is true and no interrupt is pending (`ienable & iflag == 0`):
- Return `StepStatus::Halted`
- CPU effectvely does nothing this cycle
- Will check again next cycle

**Case 2 - Wake up**: If `halted` is true and an interrupt IS pending:
- Clear `halted` flag (exit HALT mode)
- If `ime` is false: set `halt_bug = true`
- Return `StepStatus::Continue` to proceed with execution

**Why check (ienable & iflag)?** An interrupt is "pending" only if:
- A component has requested it (bit set in IF)
- AND that interrupt type is enabled (bit set in IE)

**Wake-up behavior**:
- If `IME = true`: Interrupt will be serviced immediately (next check in `step()`)
- If `IME = false`: CPU resumes normal execution, but halt_bug triggers

### The HALT Bug
```rust
fn handle_halt_bug(&mut self) {
    if self.halt_bug {
        self.pc = self.pc.wrapping_sub(1);
        self.halt_bug = false;
    }
}
```
**What is this bug?** A hardware quirk in the real Game Boy: if HALT executes with `IME = false` and an interrupt is pending, the next byte after HALT is executed twice.
**When it happens**:
1. HALT instruction executes with `IME = false`
2. An interrupt is pending (IF & IE != 0)
3. CPU wakes from HALT but doesn't service the interrupt (because IME is false)
4. The next instruction byte is fetched and executed
5. After execution, `handle_halt_bug()` decrements PC by 1
6. Next cycle: Same instruction byte is fetched again and executed

**Example**:
```
0xC000: 0x76 (HALT)
0xC001: 0x04 (INC B)

Cycle 1: Execute HALT → halted=true, PC=0xC001
Cycle 2: Wake from HALT (interrupt pending, IME=false) → halt_bug=true
Cycle 3: Execute INC B at 0xC001, PC→0xC002, then PC←0xC001
Cycle 4: Execute INC B again at 0xC001
Result: B incremented twice!
```
**Why emulate this?** Some games may rely on this behavior.

### Interrupt Management
#### handle_ime_state()
```rust
fn handle_ime_state(&mut self) -> StepStatus {
    if self.ime {
        let mut bus = self.bus.write().unwrap();
        if let Some(interrupt) = bus.interrupts_next_request() {
            self.ime = false;
            bus.interrupts_clear_request(interrupt);

            let ret_addr = self.pc;

            let sp1 = self.registers.get_sp().wrapping_sub(1);
            self.registers.set_sp(sp1);
            bus.write_byte(sp1, (ret_addr >> 8) as u8);

            let sp2 = sp1.wrapping_sub(1);
            self.registers.set_sp(sp2);
            bus.write_byte(sp2, (ret_addr & 0xFF) as u8);

            self.pc = interrupt.vector();
            StepStatus::Halted
        } else {
            StepStatus::Continue
        }
    } else {
        StepStatus::Continue
    }
}
```
**Purpose**: Checks for and services pending interrupts.
**Prerequisites**:
1. `ime` must be `true` (interrupts globally enabled)
2. An interrupt must be both requested (IF bit set) and enabled (IE bit set)
3. `interupts_next_request()` returns the highest-priority pending interrupt

#### Interrupt service sequence:
#### Step 1 - Disable further interrupts:
```rust
self.ime = false;
```
Prevents nested interrupts. The interrupt handler must use RETI to re-enable IME.

#### Step 2 - Clear the interrupt request:
```rust
bus.interrupts_clear_request(interrupt);
```
Clears the corresponding bit in IF so we don't service it again immediately.

#### Step 3 - Push return address onto stack:
```rust
let ret_addr = self.pc;

let sp1 = self.registers.get_sp().wrapping_sub(1);
self.registers.set_sp(sp1);
bus.write_byte(sp1, (ret_addr >> 8) as u8);  // High byte

let sp2 = sp1.wrapping_sub(1);
self.registers.set_sp(sp2);
bus.write_byte(sp2, (ret_addr & 0xFF) as u8);  // Low byte
```
**Why push PC?** So RETI can return to where we where.
**Stack mechanics**:
- Stack grows downward (SP decrements)
- First push high byte of PC
- Then push low byte of PC
- SP now points to low byte (top to stack)

**Example**:
- PC = 0x1234, SP = 0xFFFE
- Push high byte: SP = 0xFFFD, write 0x12 to 0xFFFD
- Push low byte: SP = 0xFFFC, write 0x34 to 0xFFFC
- Stack now: [0xFFFC] = 0x34, [0xFFFD] = 0x12

#### Step 4 - Jump to interrupt vector:
```rust
self.pc = interrupt.vector();
```
Sets PC to the interrupt handler address:
- V-Blank: 0x0040
- LCD STAT: 0x0048
- Timer: 0x0050
- Serial: 0x0058
- Joypad: 0x0060

#### Step 5 - Return status:
```rust
StepStatus::Halted
```
Tells `step()` not to execute a normal instrution this cycle. Returns 5 cycles for the interrupt handling overhead.

#### handle_ime_delay()
```rust
fn handle_ime_delay(&mut self) {
    if self.ime_delay {
        self.ime = true;
        self.ime_delay = false;
    }
}
```
**Purpose**: Implements the one-instruction delay of the EI instruction.
**Why the delay?** Hardware behavior: EI enables interrupts AFTER the next instruction executes, not immediately.
**Flow**:
1. EI instruction executes: sets `ime_delay = true` (not `ime`)
2. Next instruction executes normally
3. After that instruction: `handle_ime_delay()` activates `ime = true`
4. From this point: interrupts can be serviced

**Example**:
```
DI          ; ime = false
EI          ; ime_delay = true, ime still false
LD A, 5     ; Executes with ime=false, then handle_ime_delay() sets ime=true
ADD A, 3    ; Now interrupts can happen
```
**Why this matters**: Allows safe patterns like:
```
EI          ; Enable interrupts after next instruction
RETI        ; Return and enable interrupts (immediate)
```
Without the delay, an interrupt could fire between EI and RETI, breaking the return sequence.

### Instruction Execution
#### execute_instruction()
```rust
pub fn execute_instruction(&mut self, instruction: u8) -> u8 {
    let block = (instruction & BLOCK_MASK) >> 6;
    match block {
        0b00 => block0::execute_instruction_block0(self, instruction),
        0b01 => block1::execute_instruction_block1(self, instruction),
        0b10 => block2::execute_instruction_block2(self, instruction),
        0b11 => block3::execute_instruction_block3(self, instruction),
        _ => unreachable!(),
    }
}
```
**Purpose**: Routes instruction byte to the appropriate block handler.
**Block decoding**: Instructions are organized into 4 "blocks" based on the upper 2 bits:
```
Instruction byte: [BB][XX][XX][XX]
                   ↑
                   Block bits (6-7)
```
**BLOCK_MASK**:
```rust
const BLOCK_MASK: u8 = 0b11000000;
```
Isolates bits 6-7.

**Decoding process**:
1. `instruction & BLOCK_MASK`: Masks off lower 6 bits, keeps upper 2
2. `>> 6`: Shifts right to get 0, 1, 2 or 3
3. Match routes to appropriate block module

**Examples**:
- `0x00 = 0b00000000` -> `(0x00 & 0xC0) >> 6 = 0` → Block 0
- `0x40 = 0b01000000` -> `(0x40 & 0xC0) >> 6 = 1` → Block 1
- `0x80 = 0b10000000` -> `(0x80 & 0xC0) >> 6 = 2` → Block 2
- `0xC0 = 0b11000000` -> `(0xC0 & 0xC0) >> 6 = 3` → Block 3

**Block overview** (details in separate instructions documentation):
- **Block 0**: Miscellaneous (NOP, STOP, LD, JR, arithmetic on registers)
- **Block 1**: 8-bit loads (LD r,r) HALT
- **Block 2**: 8-bit ALU operations (ADD, SUB, AND, OR, XOR, CP)
- **Block 3**: Control flow (RET, JP, CALL), stack operations, prefix CB

### Register Access Helpers
```rust
pub fn get_r8_value(&self, register: R8) -> u8 {
    match register {
        R8::HLIndirect => {
            let addr = self.registers.get_r16_value(R16::HL);
            self.bus.read().unwrap().read_byte(addr)
        }
        _ => self.registers.get_r8_value(register),
    }
}

pub fn set_r8_value(&mut self, register: R8, value: u8) {
    match register {
        R8::HLIndirect => {
            let addr = self.registers.get_r16_value(R16::HL);
            self.bus.write().unwrap().write_byte(addr, value);
        }
        _ => self.registers.set_r8_value(register, value),
    }
}
```
**HLIndirect special case**: Most 8-bit registers (A, B, C, D, E, H, L) are in Registers struct. But (HL) - memory at address HL - requires memory access.
**get_r8_value**: If HLIndirect, reads memory at address in HL. Otherwise reads register directly.
**set_r8_value**: Same but for writes.
**Example**: LD A, (HL) with HL=0xC050
```rust
get_r8_value(R8::HLIndirect)
→ addr = registers.get_r16_value(R16::HL) // 0xC050
→ bus.read_byte(0xC050) // Returns value at that address
```

### Implementation Status
#### Completed
- ✅ Full CPU structure with all state
- ✅ Tick-based execution with proper timing
- ✅ Instruction fetch-decode-execute cycle
- ✅ HALT instruction and wake-up logic
- ✅ HALT bug implementation
- ✅ Interrupt service routine
- ✅ IME management with EI delay
- ✅ Stack operations for interrupts
- ✅ Block-based instruction routing
- ✅ Memory access through MMU bus
- ✅ Register access with (HL) special case


Last modification: 2026-01-22