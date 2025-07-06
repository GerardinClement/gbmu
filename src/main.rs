#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod app;
mod cpu;
mod gameboy;
mod mmu;
mod ppu;
mod gui;

use crate::app::App;
use std::env;
use std::fs;
use std::process;
use winit::event_loop::EventLoop;



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



use eframe::egui; // Import necessary parts of eframe and egui

fn main() -> Result<(), eframe::Error> {
    /*
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
    */
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

struct MyApp {
    label: String,
    value: f32,
    // Add a boolean field for the checkbox state
    show_extra_info: bool,
    // Add a field to store the selected color choice.
    selected_color: ColorChoice,
    // Let's add another state variable for demonstration
    counter: i32,
    // Add the current mode field
    current_mode: AppMode,
}

// Manually implement the Default trait for MyApp
impl Default for MyApp {
    fn default() -> Self {
        Self {
            label: "Initial Text".to_string(), // Use .to_string() to create a String
            value: 5.0,
            show_extra_info: false, // Default checkbox to unchecked
            selected_color: ColorChoice::Red, // Default color choice
            counter: 0, // Default counter
            current_mode: AppMode::View, // Start in View mode by default
        }
    }
}


// Replace the previous impl eframe::App for MyApp block with this enhanced version
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Central Panel for Content based on Mode ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let width = 160;
            let height = 144;
            let white_pxl = [255u8, 255,255,255];
            let mut buffer = Vec::with_capacity(width * height * 4);
            for _ in 0..(width * height) {
                buffer.extend_from_slice(&white_pxl);
            }
            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &buffer);
            let texture_handle = ctx.load_texture("gb_frame", color_image, egui::TextureOptions::default());
            ui.image(&texture_handle);
        });
    }
}


    /*

    let rom_path: String = if args.len() == 2 {
        args.pop()
            .expect("Expected a ROM name as the second argument")
    } else {
        "roms/individual/02-interrupts.gb".to_string()
    };

    let rom_data: Vec<u8> = read_rom(rom_path);

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new(rom_data);
    let _ = event_loop.run_app(&mut app);
}
    */
