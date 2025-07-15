mod app;

mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;

use gui::MyApp;

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
        "roms/dmg_boot.bin".to_string()
    };
    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}
