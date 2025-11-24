use std::{fs, process};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::app::GameApp;
use crate::displayable::UpdatableState;
use crate::ui_states::game_launched::{EmulatedGame, GameLaunchedState};
use eframe::egui;

#[derive(Default)]
pub struct StartingMenuState;

impl UpdatableState for StartingMenuState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn UpdatableState>> {
        use std::cell::RefCell;

        // Use RefCell to mutate data inside the closure
        let data = RefCell::new(None);

        egui::CentralPanel::default().show(ctx, |ui| {
            let button = ui.button("Launch Game");
            if button.clicked() {
                let (input_sender, input_receiver) = channel::<Vec<u8>>(1);
                let (image_sender, image_receiver) = channel::<Vec<u8>>(1);
                *data.borrow_mut() = Some(Box::new(GameLaunchedState {
                    emulated_game: EmulatedGame::new(
                        tokio::spawn(launch_game(
                            "roms/gb-test-roms/cpu_instrs/individual/02-interrupts.gb".to_string(),
                            input_receiver,
                            image_sender,
                        )),
                        input_sender,
                        image_receiver,
                    ),
                    actual_image: vec![0; 160 * 144 * 4],
                }) as Box<dyn UpdatableState>)
            }
        });

        data.into_inner()
    }
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
