mod app;

mod cli;
mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;
mod file;

use gui::GraphicalApp;
use crate::{cli::EmulatorArguments, file::{GbmuFile}, gui::EmulationAppOptions};
use std::sync::{LazyLock, Mutex};

static GBMU_FILE: LazyLock<Mutex<GbmuFile>> =
    LazyLock::new(|| Mutex::new(GbmuFile::get_existing_or_new()));
    
#[tokio::main]

async fn main() {

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
