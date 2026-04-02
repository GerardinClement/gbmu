mod app;

mod cpu;
mod debugger;
mod gameboy;
mod gui;
mod mmu;
mod ppu;

use gui::GraphicalApp;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut args_iterator = args.into_iter();
    _ = args_iterator.next();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    if let Some(path) = args_iterator.next() {
        if Path::new(&path).exists() {
            let _ = eframe::run_native(
                "path",
                options,
                Box::new(|_cc|
                    Box::new(
                        GraphicalApp::emulation_app(
                            String::from(path)
                        )
                    )
                )
            );
        }
    } else {
        let _ = eframe::run_native(
            "egui Demo",
            options,
            Box::new(|_cc| Box::new(GraphicalApp::default())),
        );
    }
}
