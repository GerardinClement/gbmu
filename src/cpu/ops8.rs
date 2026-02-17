use crate::cpu::registers::R8;
use crate::cpu::Cpu;
use crate::mmu::mbc::Mbc;

impl<T: Mbc> Cpu<T> {
    pub fn op_rotate_left(&mut self, target: R8, through_carry: bool, z_always_zero: bool) {
        let value = self.get_r8_value(target);

        let carry_in = if through_carry && self.registers.get_carry_flag() { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;

        let result = if through_carry {
            (value << 1) | carry_in
        } else {
            value.rotate_left(1)
        };

        self.set_r8_value(target, result);

        self.registers.set_carry_flag(carry_out);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);

        let zero = if z_always_zero { false } else {result == 0 };
        self.registers.set_zero_flag(zero);
    }

     pub fn op_rotate_right(&mut self, target: R8, through_carry: bool, z_always_zero: bool) {
        let value = self.get_r8_value(target);

        let carry_in = if through_carry && self.registers.get_carry_flag() { 1 } else { 0 };
        let carry_out = (value & 0x01) != 0;

        let result = if through_carry {
            (value >> 1) | (carry_in << 7)
        } else {
            value.rotate_right(1)
        };

        self.set_r8_value(target, result);

        self.registers.set_carry_flag(carry_out);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);

        let zero = if z_always_zero { false } else {result == 0 };
        self.registers.set_zero_flag(zero);
    }

    pub fn op_sla(&mut self, target: R8) {
        let value = self.get_r8_value(target);
        let carry_out = (value & 0x80) != 0;
        let result = value << 1;

        self.set_r8_value(target, result);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry_out);
    }

    pub fn op_sr(&mut self, target: R8, arithmetic: bool) {
        let value = self.get_r8_value(target);
        let carry_out = (value & 0x01) != 0;

        let result = if arithmetic {
            ((value as i8) >> 1) as u8 // SRA
        } else {
            value >> 1 // SRL
        };

        self.set_r8_value(target, result);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry_out);
    }

    pub fn op_swap(&mut self, target: R8) {
        let value = self.get_r8_value(target);
        let result = (value >> 4) | (value << 4);

        self.set_r8_value(target, result);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);
    }
}


#[cfg(test)]
mod tests {
    use crate::cpu::registers::{R8, R16};
    use crate::cpu::Cpu;
    use crate::mmu::mbc::RomOnly;
    use crate::mmu::Mmu;
    use std::sync::{Arc, RwLock};

    fn cpu_with_mem_at_hl(initial: u8) -> Cpu<RomOnly> {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus.clone());

        // Place HL somewhere in WRAM so boot ROM mapping can't interfere
        cpu.registers.set_r16_value(R16::HL, 0xC000);

        // Write initial value at (HL)
        bus.write().unwrap().write_byte(0xC000, initial);

        cpu
    }

    fn mem_at_hl(cpu: &Cpu<RomOnly>) -> u8 {
        let addr = cpu.registers.get_r16_value(R16::HL);
        cpu.bus.read().unwrap().read_byte(addr)
    }

    // --- ROTATES (circular) ---

    #[test]
    fn rlc_on_register_sets_carry_and_result_and_flags() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::B, 0b1000_0001);

        cpu.op_rotate_left(R8::B, false, false); // RLC B

        assert_eq!(cpu.get_r8_value(R8::B), 0b0000_0011);
        assert_eq!(cpu.registers.get_carry_flag(), true); // bit7 out
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn rrc_on_hlindirect_writes_back_to_memory() {
        let mut cpu = cpu_with_mem_at_hl(0b0000_0001);

        cpu.op_rotate_right(R8::HLIndirect, false, false); // RRC (HL)

        assert_eq!(mem_at_hl(&cpu), 0b1000_0000);
        assert_eq!(cpu.registers.get_carry_flag(), true); // bit0 out
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn rlc_sets_zero_when_result_is_zero_unless_forced() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::C, 0x00);

        cpu.op_rotate_left(R8::C, false, false); // RLC C
        assert_eq!(cpu.get_r8_value(R8::C), 0x00);
        assert_eq!(cpu.registers.get_zero_flag(), true);

        // same operation but force Z=0 (RLCA behavior style)
        cpu.op_rotate_left(R8::C, false, true);
        assert_eq!(cpu.registers.get_zero_flag(), false);
    }

    // --- ROTATES (through carry) ---

    #[test]
    fn rl_through_carry_uses_old_carry_as_bit0() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::D, 0b1000_0000);
        cpu.registers.set_carry_flag(true);

        cpu.op_rotate_left(R8::D, true, false); // RL D

        // old carry becomes bit0, old bit7 becomes carry
        assert_eq!(cpu.get_r8_value(R8::D), 0b0000_0001);
        assert_eq!(cpu.registers.get_carry_flag(), true);
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn rr_through_carry_uses_old_carry_as_bit7() {
        let mut cpu = cpu_with_mem_at_hl(0b0000_0000);
        cpu.registers.set_carry_flag(true);

        cpu.op_rotate_right(R8::HLIndirect, true, false); // RR (HL)

        assert_eq!(mem_at_hl(&cpu), 0b1000_0000);
        assert_eq!(cpu.registers.get_carry_flag(), false); // old bit0 out (was 0)
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    // --- SHIFTS ---

    #[test]
    fn sla_sets_carry_from_bit7_and_writes_back_to_hl() {
        let mut cpu = cpu_with_mem_at_hl(0b1000_0001);

        cpu.op_sla(R8::HLIndirect);

        assert_eq!(mem_at_hl(&cpu), 0b0000_0010);
        assert_eq!(cpu.registers.get_carry_flag(), true);
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn srl_shifts_in_zero_and_sets_carry_from_bit0() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::E, 0b0000_0001);

        cpu.op_sr(R8::E, false); // SRL E

        assert_eq!(cpu.get_r8_value(R8::E), 0);
        assert_eq!(cpu.registers.get_carry_flag(), true);
        assert_eq!(cpu.registers.get_zero_flag(), true);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn sra_preserves_sign_bit() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::H, 0b1000_0001);

        cpu.op_sr(R8::H, true); // SRA H

        assert_eq!(cpu.get_r8_value(R8::H), 0b1100_0000);
        assert_eq!(cpu.registers.get_carry_flag(), true); // bit0 out
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    // --- SWAP ---

    #[test]
    fn swap_swaps_nibbles_and_clears_carry() {
        let mut cpu = cpu_with_mem_at_hl(0xF0);

        cpu.registers.set_carry_flag(true); // ensure swap clears it
        cpu.op_swap(R8::HLIndirect);

        assert_eq!(mem_at_hl(&cpu), 0x0F);
        assert_eq!(cpu.registers.get_carry_flag(), false);
        assert_eq!(cpu.registers.get_zero_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }

    #[test]
    fn swap_sets_zero_when_result_is_zero() {
        let bus = Arc::new(RwLock::new(Mmu::default()));
        let mut cpu = Cpu::<RomOnly>::new(bus);

        cpu.set_r8_value(R8::A, 0x00);
        cpu.op_swap(R8::A);

        assert_eq!(cpu.get_r8_value(R8::A), 0x00);
        assert_eq!(cpu.registers.get_zero_flag(), true);
        assert_eq!(cpu.registers.get_carry_flag(), false);
        assert_eq!(cpu.registers.get_subtract_flag(), false);
        assert_eq!(cpu.registers.get_half_carry_flag(), false);
    }
}
