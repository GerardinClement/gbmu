use crate::{gui::{AppState, CoreGameDevice, DebugingDevice, EmulationDevice, WatchedAdresses}, ppu};
use eframe::egui::{ColorImage, Context, TextureOptions};


impl EmulationDevice {
    pub fn emulation_view(mut self, ctx: &Context, _frame: &mut eframe::Frame) -> AppState {
        let color_image = update_and_get_image(&mut self.core_game);
        let texture_handle = ctx.load_texture("gb_frame", color_image, TextureOptions::default());

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.image(&texture_handle);
                ui.add_space(10.0);
            });
            if ui.button("🐛 Open Debug Panel").clicked() {
                AppState::DebugingHub(self.into())
            } else {
                AppState::EmulationHub(self.into())
            }
        }).inner
    }
}

impl From<EmulationDevice> for DebugingDevice {
    fn from(original: EmulationDevice) -> Self {
        Self{
            core_game: original.core_game,
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
        }
    }
}

impl From<DebugingDevice> for EmulationDevice {
    fn from(original: DebugingDevice) -> Self {
        Self {
            core_game: original.core_game
        }
    }
}

fn update_and_get_image(game: &mut CoreGameDevice) -> ColorImage {
    let initial_width = ppu::WIN_SIZE_X;
    let initial_height = ppu::WIN_SIZE_Y;
    let scale = 3;
    let white_pxl = [255u8, 255, 255, 255];
    if let Ok(new_image) = game.image_receiver.try_recv() {
        game.actual_image = new_image;
    }
    let resized_image = scale_image(
        &game.actual_image,
        initial_width,
        initial_height,
        scale,
    );

    ColorImage::from_rgba_unmultiplied(
        [initial_width * scale, initial_height * scale],
        &resized_image,
    )
}

fn scale_image(pixels: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
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
