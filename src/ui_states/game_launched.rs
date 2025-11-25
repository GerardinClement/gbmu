use crate::app::GameApp;
use crate::ui_states::debuging_game::{DebugCommandQueries, DebugResponse, DebugedGame, DebugingGame};
use std::{fs, process};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

use eframe::egui;

use crate::displayable::UpdatableState;
use crate::ui_states::starting_menu::StartingMenuState;
use crate::displayable::NextState;

pub struct GameLaunchedState {
    pub emulated_game: EmulatedGame,
    pub actual_image: Vec<u8>,
}

pub struct EmulatedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
    pub command_sender: Sender<DebugCommandQueries>,
    pub debug_receiver: Receiver<DebugResponse>,
}

impl EmulatedGame {
    pub fn new(rom_path: String) -> Self {
        let (input_sender, input_receiver) = channel::<Vec<u8>>(1);
        let (image_sender, image_receiver) = channel::<Vec<u8>>(1);
        let (command_sender, command_query_receiver) = channel::<DebugCommandQueries>(1);
        let (debug_response_sender, debug_receiver) = channel::<DebugResponse>(10);

        let handler = tokio::spawn(launch_game(
            rom_path,
            input_receiver,
            image_sender,
            command_query_receiver,
            debug_response_sender,
        ));

        EmulatedGame {
            handler,
            input_sender,
            image_receiver,
            command_sender,
            debug_receiver,
        }
    }

    pub fn to_debuged_game(
        self
    ) -> DebugedGame {
        DebugedGame::new(
            self.handler,
            self.input_sender,
            self.image_receiver,
            self.command_sender,
            self.debug_receiver,
        )
    }
}


impl UpdatableState for GameLaunchedState {
    fn display_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<NextState> {
        use std::cell::RefCell;
        let next_state = RefCell::new(None);
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

            let button = ui.button("Activate debug mode".to_string());
            if button.clicked() {
                *next_state.borrow_mut() = Some(NextState::Debug);
            }
        });
        

        next_state.into_inner()
    }


    fn update(
        self: Box<Self>,
        next_state: NextState,
    ) -> Option<Box<dyn UpdatableState>> {
        match next_state {
            NextState::Debug => {
                Some(Box::new(DebugingGame{
                    actual_image: self.actual_image,
                    emulated_game: self.emulated_game.to_debuged_game(),
                }) as Box<dyn UpdatableState>)
            }
            _ => {
                unreachable!()
            }
        }
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

async fn launch_game(
    rom_path: String,
    input_receiver: Receiver<Vec<u8>>,
    image_sender: Sender<Vec<u8>>,
    command_receiver: Receiver<DebugCommandQueries>,
    debug_sender: Sender<DebugResponse>,
) {
    let rom_data: Vec<u8> = read_rom(rom_path);
    let app = GameApp::new(rom_data,
        input_receiver,
        image_sender,
        command_receiver, 
        debug_sender,
    );
    app.game_loop().await;
}

fn read_rom(rom_path: String) -> Vec<u8> {
    if !rom_path.is_empty() {
        match fs::read(&rom_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to read the file: {e}");
                process::exit(1)
            }
        }
    } else {
        Vec::new()
    }
}
