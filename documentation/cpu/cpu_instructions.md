# CPU Instructions Documentation
## Overview
The Game Boy CPU has 256 possible opcodes (0x00-0xFF), plus 256 extended opcodes accessed via the CV prefix (0xCB). Instructions are organized into "blocks" based on bit patterns, making decoding systematic. This document explains the instruction organization, decoding strategy, and provides examples from each category.

## Instruction Organization

### Block-Based Decoding
Instructions are divided into 4 main blocks based on bits 6-7 (the two most significant bits):
```
Instruction byte: [BB][XXXXXX]
                   ↑
                   Blocks 0-3
```
- **Block 0** (00xxxxxx): Miscellaneous. Loads, jumps, arithmetic on 16-bit registers
- **Block 1** (01xxxxxx): 8-bit register loads (LD r, r) and HALT
- **Block 2** (10xxxxxx): 8-bit ALU operations on register A
- **Block 3** (11xxxxxx): Control flow, stack operations, immediate ALU, I/O, prefix CB

### CB Prefix Block
Opcode 0xCB is special: it's followed by another byte that specifies bit manipulation operations (rotates, shifts, bit test/set/reset).

## Supporting Structures
### Registers (R8 and R16)
#### R8 - 8-bit Registers:
```rust
pub enum R8 {
    B = 0, C = 1, D = 2, E = 3,
    H = 4, L = 5, HLIndirect = 6, A = 7,
}
```
Registers can be accessed individually or paired:
- **Pairs**: BC, DE, HL, AF (A + Flags)
- **HLIndirect**: Special case - represents memory at address [HL], not a register

#### R16 - 16-bit Register Pairs:
```rust
pub enum R16 {
    BC = 0, DE = 1, HL = 2, SP = 3,
}
```
Formed by combining two 8-bit registers (B+C, D+E, H+L) plus the Stack Pointer.

#### Flag Register
The F register stores 4 flags in its upper nibble:
```
Bit 7: Zero (Z)         - Set if result is 0
Bit 6: Subtract (N)     - Set if last operation was subtraction
Bit 5: Half Carry (H)   - Set if carry from bit 3 to 4
Bit 4: Carry (C)        - Set if carry/borrow occurred
Bits 0-3: Unused (always 0)
```
#### Flag operations:
- Flags are automatically set by arithmetic/logic operations
- Can be tested by conditional instructions (JP, JR, CALL, RET)
- Stored as bool internally, converted to/from u8 when needed

#### Conditions
```rust
pub enum Cond {
    NZ = 0,  // Not Zero (Z flag = 0)
    Z = 1,   // Zero (Z flag = 1)
    NC = 2,  // No Carry (C flag = 0)
    C = 3,   // Carry (C flag = 1)
    None = 4, // Always true (unconditional)
}
```
Used by conditional jumps, calls, and returns.

### Block 0: Miscellaneous Operations
**Pattern**: `00xxxxxx`
**Categories**:
1. **NOP** (0x00): Do nothing
2. **16-bit loads**: LD r16, imm16 (load immediate 16-bit value)
3. **Indirect loads**: LD [BC], A or LD A, [BC] (memory access via register pair)
4. **16-bit arithmetic**: INC/DEC r16, ADD HL, r16
5. **8-bit arithmetic**: INC/DEC r8
6. **8-bit immediate load**: LD r8, imm8
7. **Rotates on A**: RLCA, RRCA, RLA, RRA
8. **Special**: DAA, CPL, SCF, CCF
9. **Jumps**: JR (relative jump), JR cond, imm8
10. **Control**: STOP

#### Decoding Strategy
Block 0 uses complex decoding due to many instruction types. The `get_instruction_block0()` function:
1. Checks for exact matches first
2. Uses bit masks on lower 3-4 bits to identify instruction families
3. Combines multipl masks when needed to disambiguate

#### Example Instructions
#### NOP (0x00)
```rust
fn noop(cpu: &mut Cpu) -> u8 {
    cpu.pc += 1;
    4 // 4 cycles
}
```
Does nothing, advances PC.

#### LD BC, imm16 (0x01):
```rust
fn load_r16_imm16(cpu: &mut Cpu, instruction: u8) -> u8 {
    let imm16 = utils::get_imm16(cpu);  // Read next 2 bytes
    let r16 = R16::from((instruction & utils::R16_MASK) >> 4);
    cpu.registers.set_r16_value(r16, imm16);
    cpu.pc = cpu.pc.wrapping_add(3);  // Opcode + 2 bytes
    12  // 12 cycles
}
```
Bits 4-5 encode which register pair: BC=0, DE=1, HL=2, SP=3.

#### INC B (0x04):
```rust
fn inc_r8(cpu: &mut Cpu, instruction: u8) -> u8 {
    let r8 = utils::convert_dest_index_to_r8(instruction);
    let value = cpu.get_r8_value(r8);
    let new_value = value.wrapping_add(1);
    
    cpu.registers.set_zero_flag(new_value == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag((value & 0x0F) + 1 > 0x0F);
    
    cpu.set_r8_value(r8, new_value);
    cpu.pc = cpu.pc.wrapping_add(1);
    4
}
```
Bits 3-5 encode target register. Sets flags appropriately.

#### JR cond, imm8 (0x20, 0x28, 0x30, 0x38):
```rust
fn jr(cpu: &mut Cpu, instruction: u8, has_cond: bool) -> u8 {
    if has_cond {
        let cond = convert_index_to_cond(instruction);
        if !cond.test(&mut cpu.registers) {
            cpu.pc = cpu.pc.wrapping_add(2);
            return 8;  // Condition failed: 8 cycles
        }
    }
    let offset = cpu.bus.read().unwrap().read_byte(cpu.pc + 1) as i8;
    cpu.pc = ((cpu.pc as i32) + 2 + (offset as i32)) as u16;
    12  // Condition passed or unconditional: 12 cycles
}
```
Relative jump with signed 8-bit offset. Conditional versions check flags first.

### Block 1: 8-bit Register Loads
**Pattern**: `01xxxxxx`
**Primary instruction**: LD r8, r8 (load register to register)
#### Encoding
```
01[DDD][SSS]
   ↑    ↑
   Dest Source
```
Both dest and source use same 3-bit encoding:
- 000=B, 001=C, 010=D, 011=E, 100=H, 101=L, 110=[HL], 111=A

**Special case**: 0x76 (01110110) would be LD [HL], [HL] but is actually HALT.

#### Implementation
```rust
fn load_r8_r8(cpu: &mut Cpu, instruction: u8) -> u8 {
    let source: R8 = utils::convert_source_index_to_r8(instruction);
    let dest: R8 = utils::convert_dest_index_to_r8(instruction);
    
    let value = cpu.get_r8_value(source);
    cpu.set_r8_value(dest, value);
    cpu.pc = cpu.pc.wrapping_add(1);
    
    if source == R8::HLIndirect { 8 } else { 4 }
}
```
**Timing**: 4 cycles normally, 8 cycles if reading from [HL] (memory access).
**Examples**:
- 0x40: LD B, B (copy B to itself - valid but pointless)
- 0x47: LD B, A
- 0x7E: LD A, [HL] (load from memory at address HL)
- 0x76: HALT (exception to the pattern)

### HALT
```rust
fn halt(cpu: &mut Cpu) -> u8 {
    cpu.halted = true;
    cpu.pc = cpu.pc.wrapping_add(1);
    4
}
```
Sets halted flag. CPU will stop executing until interrupt pending.

### Block 2: 8-bit ALU Operations
**Pattern**: `10[000][SSS]`
All operations are on register A with another register/[HL] as source.

#### Operations (OOO)
- **000**: ADD A, r8 (addition)
- **001**: ADC A, r8 (add with carry)
- **010**: SUB A, r8 (add with carry)
- **011**: SBC A, r8 (subtract with carry)
- **100**: AND A, r8 (bitwise AND)
- **101**: XOR A, r8 (bitwise XOR)
- **110**: OR A, r8 (bitwise OR)
- **111**: CP A, r8 (compare - subtract but don't store result)

#### Source Encoding (SSS)
Same as Block 1: 000=B, 001=C, ..., 110=[HL], 111=A

### Example: ADD A,B (0x80)
```rust
fn add_a_r8(cpu: &mut Cpu, instruction: u8, with_carry: bool) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let r8_value = cpu.get_r8_value(r8);
    
    cpu.registers.add_to_r8(R8::A, r8_value, with_carry);
    cpu.pc = cpu.pc.wrapping_add(1)
}
```
The `add_to_r8` method handles flag settings:
```rust
pub fn add_to_r8(&mut self, target: R8, value: u8, with_carry: bool) {
    let target_value = self.r8[target as usize];
    let carry_in = if with_carry && self.get_carry_flag() { 1 } else { 0 };
    
    let (intermediate, carry1) = target_value.overflowing_add(value);
    let (result, carry2) = intermediate.overflowing_add(carry_in);
    
    self.r8[target as usize] = result;
    
    let half_carry = ((target_value & 0x0F) + (value & 0x0F) + carry_in) > 0x0F;
    let carry = carry1 || carry2;
    
    self.f.set_all(result == 0, false, half_carry, carry);
}
```
**Flag calculation**:
- **Zero**: Result is 0
- **Subtract**: False (this is addition)
- **Half Carry**: Carry from bit 3 to bit 4
- **Carry**: Overflow occured

#### Example: CP A,C (0xB9)
Compare is subtract without storing result. Only flags are affected.
```rust
fn cp_a_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let r8_value = cpu.get_r8_value(r8);
    let a_value = cpu.get_r8_value(R8::A);
    
    let value = a_value.wrapping_sub(r8_value);
    
    cpu.registers.set_zero_flag(value == 0);  // A == r8
    cpu.registers.set_subtract_flag(true);
    cpu.registers.set_half_carry_flag((a_value & 0x0F) < (r8_value & 0x0F));
    cpu.registers.set_carry_flag(r8_value > a_value);  // A < r8
    
    cpu.pc = cpu.pc.wrapping_add(1)
}
```
**Timing**: 4 cycles normally, 8 cycles if source is [HL].

### Block 3: Control Flow and Extended Operations
**Pattern**: `11xxxxxx`
Most complex block with many instruction types.

#### Categories
1. **ALU with immediate**: ADD A, imm8 / SUB A, imm8 / etc.
2. **Conditional returns**: RET cond
3. **Unconditional return**: RET, RETI
4. **Jumps**: JP cond, imm16 / JP imm16 / JP HL
5. **Calls**: CALL cond, imm16 / CALL imm16
6. **Stack operations**: PUSH r16 / POP r16
7. **Restarts**: RST n (call to fixed addresses)
8. **I/O operations**: LDH [imm8], A / LDH A, [imm8]
9. **Special loads**: LD [imm16], A / LD A, [imm16]
10. **SP operations**: ADD SP, imm8 / LD HL, SP+imm8 / LD SP, HL
11. **Interrupt control**: DI, EI
12. **Prefix**: CB (extended instructions)

#### Example: CALL imm16(0xCD)
```rust
fn call_imm16(cpu: &mut Cpu, instruction: u8, with_cond: bool) -> u8 {
    let cond = if with_cond {
        utils::convert_index_to_cond(instruction)
    } else {
        Cond::None
    };
    
    let imm16 = utils::get_imm16(cpu);
    
    if cpu.registers.check_condition(cond) || !with_cond {
        cpu.registers.push_sp(
            &mut cpu.bus.write().unwrap(),
            cpu.pc.wrapping_add(3)  // Return address after this instruction
        );
        cpu.pc = imm16;
        20  // Condition passed: 20 cycles
    } else {
        cpu.pc = cpu.pc.wrapping_add(3);
        12  // Condition failed: 12 cycles
    }
}
```
CALL pushes return address onto stack then jumps to target address.

#### Example: PUSH BC (0xC5)
```rust
fn push_r16(cpu: &mut Cpu, instruction: u8) -> u8 {
    let r16 = utils::convert_index_to_r16(instruction);
    let value = cpu.registers.get_r16_value(r16);
    cpu.registers.push_sp(&mut cpu.bus.write().unwrap(), value);
    cpu.pc = cpu.pc.wrapping_add(1);
    16
}

// push_sp implementation in registers.rs:
pub fn push_sp(&mut self, bus: &mut Mmu, value: u16) {
    let low = (value & 0x00FF) as u8;
    let high = (value >> 8) as u8;
    self.sp = self.sp.wrapping_sub(1);
    bus.write_byte(self.sp, high);  // High byte first
    self.sp = self.sp.wrapping_sub(1);
    bus.write_byte(self.sp, low);   // Then low byte
}
```
Stack grows downward. High byte pushed first (big-endian on stack).

#### Example: EI (0xFB)
```rust
fn ei(cpu: &mut Cpu) -> u8 {
    cpu.ime_delay = true;  // IME will be enabled after next instruction
    cpu.pc = cpu.pc.wrapping_add(1);
    4
}
```
Enable interrupts with one-instruction delay.

### CB Prefix Block: Bit Operations
**Access**: Two-byte instruction: 0xCB followed by actual opcode.

#### Categories
1. **Rotates**: RLC, RRC, RL, RR (all 8 registers)
2. **Shifts**: SLA, SRA, SRL (arithmetic/logical shifts)
3. **Swap**: SWAP (swap high/low nibbles)
4. **Bit test**: BIT b, r8 (test if bit b is set)
5. **Bit reset**: RES b, r8 (clear bit b)
6. **Bit set**: SET b, r8 (set bit b)

#### Encoding Patterns
**Rotates/Shifts/Swap** (CB 00-3F):
```
CB [00][OOO][RRR]
        ↑    ↑
        Op   Register
```
- OOO: 000=RLC, 001=RRC, 010=RL, 011=RR, 100=SLA, 101=SRA, 110=SWAP, 111=SRL
- RRR: 000=B, 001=C, ..., 110=[HL], 111=A

**Bit operations** (CB 40-FF):
```
CB [PP][BBB][RRR]
     ↑   ↑    ↑
     Op  Bit  Register
```
- PP: 01=BIT, 10=RES, 11=SET
- BBB: Bit number 0-7
- RRR: Register encoding

#### Example: BIT 7, H (CB 0x7C)
```rust
fn bit_b3_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let b3 = (instruction & B3_MASK) >> 3;  // Extract bit number
    let r8_value = cpu.get_r8_value(r8);
    
    cpu.registers.set_zero_flag((r8_value & (1 << b3)) == 0);
    cpu.registers.set_subtract_flag(false);
    cpu.registers.set_half_carry_flag(true);
    
    cpu.pc = cpu.pc.wrapping_add(2);  // CB prefix + opcode
}
```
Tests if bit b3 is 0. Sets Zero flag accordingly.

#### Example: SET 3, L (CB 0xDD)
```rust
fn set_b3_r8(cpu: &mut Cpu, instruction: u8) {
    let r8: R8 = utils::convert_source_index_to_r8(instruction);
    let b3 = (instruction & B3_MASK) >> 3;
    let mut r8_value = cpu.get_r8_value(r8);
    
    r8_value |= 1 << b3;  // Set bit b3 to 1
    cpu.set_r8_value(r8, r8_value);
    
    cpu.pc = cpu.pc.wrapping_add(2);
}
```
**Timing**: CB instructions take 8 cycles, or 16 cycles if operating on [HL].

### Utility Functions
#### Register Extraction
```rust
pub fn convert_source_index_to_r8(instruction: u8) -> R8 {
    let r8_index: u8 = instruction & SOURCE_R8_MASK;  // Lower 3 bits
    R8::from(r8_index)
}

pub fn convert_dest_index_to_r8(instruction: u8) -> R8 {
    let r8_index: u8 = (instruction & DEST_R8_MASK) >> 3;  // Bits 3-5
    R8::from(r8_index)
}
```
Extract register encoding from instruction bits.

#### Immediate Value Reading
```rust
pub fn get_imm16(cpu: &mut Cpu) -> u16 {
    let lsb = cpu.bus.read().unwrap().read_byte(cpu.pc + 1) as u16;
    let msb = cpu.bus.read().unwrap().read_byte(cpu.pc + 2) as u16;
    (msb << 8) | lsb  // Little-endian
}
```
16-bit immediates stored as two bytes: low byte first, then high byte.

#### Condition Testing
```rust
impl Cond {
    pub fn test(&self, registers: &mut Registers) -> bool {
        match self {
            Cond::NZ => !registers.get_zero_flag(),
            Cond::Z => registers.get_zero_flag(),
            Cond::NC => !registers.get_carry_flag(),
            Cond::C => registers.get_carry_flag(),
            Cond::None => true,
        }
    }
}
```

### Instruction Timing
Different instructions take different numbers of cycles:
- **Simple operations**: 4 cycles (register operations)
- **Memory access**: 8 cycles (loading from [HL])
- **16-bit operations**: 8-12 cycles
- **Jumps/Calls**: 12-20 cycles (more if condition passes)
- **CB prefix**: 8-16 cycles
- **Stack operations**: 12-16 cycles

Conditional instructions have different timing based on whether condition passes or fails.



last modification: 2026/01/23