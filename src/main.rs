#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod app;
mod cpu;
mod gameboy;
mod mmu;
mod ppu;

use crate::app::App;
use std::env;
use std::fs;
use std::process;
use winit::event_loop::EventLoop;

fn read_rom(rom_path: String) -> Vec<u8> {
    if !rom_path.is_empty() {
        match fs::read(&rom_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to read the file: {e}");
                process::exit(1);
            }
        }
    } else {
        Vec::new()
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let rom_path: String = if args.len() == 2 {
        args.pop()
            .expect("Expected a ROM name as the second argument")
    } else {
        "roms/cpu_instrs.gb".to_string()
    };

    let rom_data: Vec<u8> = read_rom(rom_path);

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new(rom_data);
    let _ = event_loop.run_app(&mut app);
}
