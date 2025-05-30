mod cpu;
mod instructions;
mod memory;

use crate::cpu::cpu::CPU;


fn main() {
    let cpu = CPU::default();
    println!("{}", cpu);
}
