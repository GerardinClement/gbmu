mod registers;
mod flags_registers;
mod cpu;
mod instructions;

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

    let cpu = CPU::default();
    println!("{}", cpu);
}
