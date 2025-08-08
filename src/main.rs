#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod cpu;
mod gameboy;
mod mmu;
mod ppu;
mod gui;
mod app;

use gui::app::MyApp;



#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}
