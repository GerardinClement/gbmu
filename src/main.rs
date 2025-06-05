#![allow(unused_variables)]
#![allow(dead_code)]

mod cpu;
mod memory;

use crate::cpu::Cpu;

fn main() {
    let cpu = Cpu::default();
    println!("{}", cpu);
}
