mod registers;
mod flags_registers;
mod cpu;
mod instruction;
mod memory;

use registers::Registers;

use crate::flags_registers::FlagsRegister;
use crate::cpu::CPU;


fn main() {
    let flags = FlagsRegister {
        zero: false,
        subtract: false,
        half_carry: false,
        carry: false,
    };

    let mut cpu = CPU::default();
    cpu.registers.a = cpu.add(5);
    println!("{}", cpu);
}
