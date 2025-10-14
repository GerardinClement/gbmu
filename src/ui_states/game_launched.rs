use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use eframe::egui;

use crate::displayable::UpdatableState;

pub struct GameLaunchedState {
    pub emulated_game: EmulatedGame,
    pub actual_image: Vec<u8>,
}

pub struct EmulatedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
}

impl EmulatedGame {
    pub fn new(
        handler: JoinHandle<()>,
        input_sender: Sender<Vec<u8>>,
        image_receiver: Receiver<Vec<u8>>,
    ) -> Self {
        EmulatedGame {
            handler,
            input_sender,
            image_receiver,
        }
    }
}

impl UpdatableState for GameLaunchedState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn UpdatableState>> {
        egui::CentralPanel::default().show(ctx, |ui| {
            let initial_width = 160;
            let initial_height = 144;
            let scale = 3;
            let white_pxl = [255u8, 255, 255, 255];
            if let Ok(new_image) = self.emulated_game.image_receiver.try_recv() {
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
        });
        None
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
