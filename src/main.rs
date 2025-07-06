#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod app;
mod cpu;
mod gameboy;
mod mmu;
mod ppu;

use app::GameApp;
use eframe::egui;
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
) {
    let rom_data: Vec<u8> = read_rom(rom_path);
    let mut app = GameApp::new(rom_data);
    loop {
        let buffer = app.update();
        if let Some(image) = buffer {
            _ = image_sender.send(image).await;
        }
    }
}

fn emulation_button(ui: &mut egui::Ui) -> Option<EmulatedGame> {
    // Put the buttons and label on the same row:
    let button = ui.button("ceci est un bouton");
    if button.clicked() {
        let (input_sender, input_receiver) = channel::<Vec<u8>>(1);
        let (image_sender, image_receiver) = channel::<Vec<u8>>(1);
        Some(EmulatedGame {
            input_sender,
            image_receiver,
            handler: tokio::spawn(launch_game(
                "gb-test-roms/cpu_instrs/individual/02-interrupts.gb".to_string(),
                input_receiver,
                image_sender,
            )),
        })
    } else {
        None
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Central Panel for Content based on Mode ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(game) = &mut self.emulated_game {
                let width = 160;
                let height = 144;
                let white_pxl = [255u8, 255, 255, 255];
                if let Ok(new_image) = game.image_receiver.try_recv() {
                    self.actual_image = new_image;
                }
                let color_image =
                    egui::ColorImage::from_rgba_unmultiplied([width, height], &self.actual_image);
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

struct EmulatedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
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
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

/*

}
    */
