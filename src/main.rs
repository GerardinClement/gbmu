#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod app;
mod cpu;
mod debugger;
mod displayable;
mod gameboy;
mod mmu;
mod ppu;
mod ui_states;

use eframe::egui;

use crate::displayable::UpdatableState;

use ui_states::starting_menu::StartingMenuState;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(new_app_state) = self.app_state.display_gui(ctx, _frame) {
            let last_state = std::mem::replace(&mut self.app_state, Box::new(StartingMenuState));
            if let Some(new_state) = last_state.update(new_app_state) {
                let _ = std::mem::replace(&mut self.app_state, new_state);
            }
            
        }
        ctx.request_repaint();
    }
}

impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            app_state: Box::new(StartingMenuState),
        }
    }
}

struct MyApp {
    app_state: Box<dyn UpdatableState>,
}

#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]) // Taille par défaut plus grande
            .with_min_inner_size([800.0, 600.0]) // Taille minimum
            .with_resizable(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}
