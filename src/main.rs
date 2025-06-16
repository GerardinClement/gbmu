#![allow(unused_variables)]
#![allow(dead_code)]

mod cpu;
mod gameboy;
mod memory;

use std::env;
use std::fs;
use std::process;

fn read_rom(rom_path: String) -> Vec<u8> {
    let rom_data = if !rom_path.is_empty() {
        match fs::read(&rom_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to read the file: {}", e);
                process::exit(1);
            }
        }
    } else {
        Vec::new()
    };
    rom_data
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let rom_path: String = if args.len() == 2 {
        args.pop()
            .expect("Expected a ROM name as the second argument")
    } else {
        "roms/individual/03-op sp,hl.gb".to_string()
    };

    let rom_data: Vec<u8> = read_rom(rom_path);

    let mut gameboy = gameboy::GameBoy::new(rom_data);
    println!("{}", gameboy.cpu);

    gameboy.run();
}
