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

/*<<<<<<< HEAD
use app::GameApp;
use eframe::egui;
use debugger::debbuger::*;
use eframe::egui::ColorImage;
use std::fs;
use std::process;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;
======= */
use eframe::egui;

use crate::displayable::UpdatableState;

/*
impl MyApp {
    fn update_frame(&mut self) -> Option<ColorImage> {
        if let Some(game) = &mut self.emulated_game {
            let initial_width = 160;
            let initial_height = 144;
            let scale = 3;
            let white_pxl = [255u8, 255, 255, 255];
            if let Ok(new_image) = game.image_receiver.try_recv() {
                self.actual_image = new_image;
            }
            let resized_image =
                double_size_image(&self.actual_image, initial_width, initial_height, scale);

            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [initial_width * scale, initial_height * scale],
                &resized_image,
            );
            Some(color_image)
        } else {
            None
        }
    }
}
=======
*/
use ui_states::starting_menu::StartingMenuState;


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
/*
<<<<<<< HEAD
        let color_image = self.update_frame();
        let texture_handle = color_image
            .map(|image| ctx.load_texture("gb_frame", image, egui::TextureOptions::default()));

        if let Some(game) = &mut self.emulated_game {
            update_info_struct(game);
            if game.is_debug {
                egui::SidePanel::right("debug_panel")
                    .resizable(true)
                    .default_width(400.0)
                    .min_width(300.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Debug Panel");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("✖ Close").clicked() {
                                            game.is_debug = false;
                                        }
                                    },
                                );
                            });
                            ui.separator();

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Step Control").strong());
                                step_mode_button(ui, game);
                                step_button(ui, game);
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Registers").strong());
                                get_registers(ui, game);
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Next Instructions").strong());
                                get_next_instructions(ui, game);
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Memory Watch").strong());
                                watch_address(ui, game);
                            });
                        });
                    });
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(game) = &mut self.emulated_game {
                let initial_width = 160;
                let initial_height = 144;
                let scale = 3;
                let white_pxl = [255u8, 255, 255, 255];
                if let Ok(new_image) = game.image_receiver.try_recv() {
                    self.actual_image = new_image;
                }

                let resized_image =
                    double_size_image(&self.actual_image, initial_width, initial_height, scale);

                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [initial_width * scale, initial_height * scale],
                    &resized_image,
                );
                let texture_handle =
                    ctx.load_texture("gb_frame", color_image, egui::TextureOptions::default());
                ui.image(&texture_handle);
            } else {
                self.emulated_game = emulation_button(ui);
            }
        });

=======
*/
        if let Some(new_app_state) = self.app_state.update(ctx, _frame) {
            self.app_state = new_app_state;
        }
        ctx.request_repaint();
    }
}

/*
pub struct EmulatedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
    command_query_sender: Sender<DebugCommandQueries>,
    debug_response_receiver: Receiver<DebugResponse>,

    /*
    Info stored for the GUI to use them;
    These are the responses from the sending/receiving operation
    */

=======
>>>>>>> 862f237 (refactor: refactor de la partie egui)
*/
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
