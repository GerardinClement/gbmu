mod app;

mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;

use gui::GraphicalApp;

#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(GraphicalApp::default())),
    );
}
