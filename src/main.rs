#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod app;
mod cpu;
mod debugger;
mod gameboy;
mod mmu;
mod ppu;

use app::GameApp;
use debugger::debbuger::*;
use eframe::egui;
use eframe::egui::ColorImage;
use std::fs;
use std::process;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

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

#[derive(Debug, PartialEq, Clone, Copy)]
enum ColorChoice {
    Red,
    Green,
    Blue,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum AppMode {
    View,
    Edit,
    Settings,
}

async fn launch_game(
    rom_path: String,
    input_receiver: Receiver<Vec<u8>>,
    image_sender: Sender<Vec<u8>>,
    command_query_receiver: Receiver<DebugCommandQueries>,
    debug_response_sender: Sender<DebugResponse>,
) {
    let rom_data: Vec<u8> = read_rom(rom_path);
    let mut app = GameApp::new(rom_data, command_query_receiver, debug_response_sender);

    loop {
        let buffer = app.update();
        if let Some(image) = buffer {
            _ = image_sender.send(image).await;
        }
    }
}

pub struct WatchedAdresses {
    addresses_n_values: Vec<(u16, u16)>,
}

pub enum DebugCommandQueries {
    SetDebugMode,
    SetStepMode,
    ExecuteInstruction(u8),
    ExecuteNextInstructions(usize),
    GetNextInstructions(u8),
    GetRegisters,
    WatchAddress(u16),
    GetAddresses,
}

pub enum DebugResponse {
    DebugModeSet(bool),
    StepModeSet(bool),
    InstructionsExecuted(usize),
    NextInstructions(Vec<u16>),
    AddressesWatched(WatchedAdresses),
    Registers(u8, u8, u8, u8, u8, u8, u8, u16, u16),
}

fn emulation_button(ui: &mut egui::Ui) -> Option<EmulatedGame> {
    // Put the buttons and label on the same row:
    let button = ui.button("ceci est un bouton");
    if button.clicked() {
        let (input_sender, input_receiver) = channel::<Vec<u8>>(1);
        let (image_sender, image_receiver) = channel::<Vec<u8>>(1);
        let (command_query_sender, command_query_receiver) = channel::<DebugCommandQueries>(1);
        let (debug_response_sender, debug_response_receiver) = channel::<DebugResponse>(10);
        Some(EmulatedGame {
            input_sender,
            image_receiver,
            command_query_sender,
            debug_response_receiver,
            handler: tokio::spawn(launch_game(
                "roms/gb-test-roms/cpu_instrs/cpu_instrs.gb".to_string(),
                input_receiver,
                image_sender,
                command_query_receiver,
                debug_response_sender,
            )),
            next_instructions: Vec::new(),
            watched_adress: WatchedAdresses {
                addresses_n_values: Vec::new(),
            },
            registers: (0, 0, 0, 0, 0, 0, 0, 0, 0),
            is_debug: false,
            is_step: false,
            watched_address_value: 0,
            nb_instruction: 0,
            error_message: None,
            hex_string: String::new(),
        })
    } else {
        None
    }
}

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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        ctx.request_repaint();
    }
}

fn double_size_image(pixels: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
    let scale_w = width * scale;
    let scale_h = height * scale;
    let size = scale_h * scale_w;

    (0..size)
        .map(|index| {
            let y = index / scale_w;
            let x = index % scale_w;
            let orig_y = y / scale;
            let orig_x = x / scale;
            let index_to_copy = (orig_y * width + orig_x) * 4;
            &pixels[index_to_copy..index_to_copy + 4]
        })
        .flat_map(|slice| slice.iter().copied())
        .collect()
}

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
    next_instructions: Vec<u16>,
    watched_adress: WatchedAdresses,
    registers: (u8, u8, u8, u8, u8, u8, u8, u16, u16),
    is_debug: bool,
    is_step: bool,
    watched_address_value: u16,
    nb_instruction: u8,
    error_message: Option<String>,
    hex_string: String,
}

impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            emulated_game: None,
            actual_image: vec![0; 160 * 144 * 4],
        }
    }
}

struct MyApp {
    emulated_game: Option<EmulatedGame>,
    actual_image: Vec<u8>,
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
