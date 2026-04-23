mod app;

mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;

use clap::{Arg, ArgAction, command};
use gui::GraphicalApp;

use crate::gui::EmulationAppOptions;

#[tokio::main]
async fn main() {
    let start_options = get_matches();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    let app = if let Some(rom_path) = start_options.rom_path {
        let options = EmulationAppOptions::new(
            rom_path,
            start_options.boot_rom
        );
        GraphicalApp::create_emulation_app(options)
    } else {
        GraphicalApp::default()
    };

    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}

struct StartOptions {
    rom_path: Option<String>,
    boot_rom: bool,
}

fn get_matches() -> StartOptions {
    let matches = command!()
        .arg(
            Arg::new( "rom_path")
                .help("The path of the rom you want to launch.")
        )
        .arg(
            Arg::new("boot_rom")
                .short('b')
                .long("boot_rom")
                .action(ArgAction::SetTrue)
                .required(false)
                .help("If set, nintendo basic boot rom will boot first.")
        )
        .get_matches();


    let rom_path = if let Some(path) = matches.get_one::<String>("rom_path") {
        Some(String::from(path))
    } else {
        None
    };

    let boot_rom = matches.get_flag("boot_rom");

    return StartOptions {
        rom_path,
        boot_rom,
    }
}
