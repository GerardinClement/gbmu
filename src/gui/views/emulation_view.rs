use crate::gui::{
        AppState, CoreGameDevice, DebugingDevice, EmulationDevice, SelectionDevice, WatchedAdresses
    };
use eframe::egui::{Context, ahash::HashSet};
use std::thread::sleep;

use std::sync::atomic::Ordering;

use std::time::{Instant, Duration};

impl EmulationDevice {
    pub fn emulation_view(mut self, ctx: &Context, _frame: &mut eframe::Frame) -> AppState {
        let debut = Instant::now();
        self.core_game.update_and_size_image(ctx);
        let duration = debut.elapsed();
        //println!("update and size image: Temps écoulé : {:?} ({} ms)", duration, duration.as_millis());
        let input = self.core_game.capture_input(ctx);

        self.core_game.input_sender.send(input);

        eframe::egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if let Some(texture) = self.core_game.sized_image {
                        ui.image(texture);
                    }
                    ui.add_space(10.0);
                });
                if ui.button("🐛 Open Debug Panel").clicked() {
                    AppState::DebugingHub(self.into())
                } else {
                    AppState::EmulationHub(self)
                }
            })
            .inner
    }
}

impl From<EmulationDevice> for DebugingDevice {
    fn from(original: EmulationDevice) -> Self {
        original
            .core_game
            .global_is_debug
            .fetch_xor(true, Ordering::Relaxed);
        Self {
            core_game: original.core_game,
            next_instructions: Vec::new(),
            watched_adress: WatchedAdresses {
                addresses_n_values: Vec::new(),
            },
            registers: (0, 0, 0, 0, 0, 0, 0, 0, 0),
            is_step: false,
            watched_address_value: 0,
            nb_instruction: 0,
            error_message: None,
            hex_string: String::new(),
        }
    }
}

impl From<SelectionDevice> for EmulationDevice {
    fn from(original: SelectionDevice) -> Self {
        let path = original.path;
        let core_game = CoreGameDevice::new(path);
        EmulationDevice { core_game }
    }
}

impl From<DebugingDevice> for EmulationDevice {
    fn from(original: DebugingDevice) -> Self {
        original
            .core_game
            .global_is_debug
            .fetch_xor(true, Ordering::Relaxed);
        Self {
            core_game: original.core_game,
        }
    }
}


pub fn scale_image(pixels: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
    let scale_w = width * scale;
    let scale_h = height * scale;
    let size = scale_h * scale_w;

    (0..size)
        .map(|index| {
            let y = index / scale_w;
            let x = index % scale_w;
            let orig_y = y / scale;
            let orig_x = x / scale;
            let index_to_copy = (orig_y * width + orig_x) * 3;
            &pixels[index_to_copy..index_to_copy + 3]
        })
        .flat_map(|slice| slice.iter().copied())
        .collect()
}
