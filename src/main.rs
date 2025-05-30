mod registers;
mod flags_registers;
mod cpu;
mod instructions;

use registers::Registers;
use crate::cpu::CPU;


fn main() {
    let cpu = CPU::default();
    println!("{}", cpu);
}
