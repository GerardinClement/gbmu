mod app;

mod cli;
mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;
mod file;

use std::process::exit;

use gui::GraphicalApp;

use file::GmbuFile;

use crate::{cli::EmulatorArguments, gui::EmulationAppOptions};

#[tokio::main]
async fn main() {

    let gbmu_file = GmbuFile::get_existing_or_new();
    
    let arguments = EmulatorArguments::get();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    let app = if let Some(rom_path) = arguments.rom_path {
        let options = EmulationAppOptions::new(
            rom_path,
            arguments.boot_rom
        );
        GraphicalApp::create_emulation_app(options)
    } else {
        GraphicalApp::default()
    };

    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    );
}
