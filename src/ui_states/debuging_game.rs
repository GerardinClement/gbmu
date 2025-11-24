use tokio::task::JoinHandle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::displayable::UpdatableState;


pub struct DebugingGame {
    pub emulated_game: DebugedGame,
    pub actual_image: Vec<u8>,
}

pub struct DebugedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
    debug_sender: Sender<DebugCommandQueries>,
    debug_receiver: Receiver<DebugResponse>
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

pub struct WatchedAdresses {
    addresses_n_values: Vec<(u16, u16)>,
}

pub enum DebugResponse {
    DebugModeSet(bool),
    StepModeSet(bool),
    InstructionsExecuted(usize),
    NextInstructions(Vec<u16>),
    AddressesWatched(WatchedAdresses),
    Registers(u8, u8, u8, u8, u8, u8, u8, u16, u16),
}

impl DebugedGame {
    pub fn new(
        handler: JoinHandle<()>,
        input_sender: Sender<Vec<u8>>,
        image_receiver: Receiver<Vec<u8>>,
        debug_sender: Sender<DebugCommandQueries>,
        debug_receiver: Receiver<DebugResponse>
    ) -> Self {
        DebugedGame {
            handler,
            input_sender,
            image_receiver,
            debug_sender,
            debug_receiver
        }
    }
}

impl UpdatableState for DebugingGame {
    fn update(
            &mut self,
            ctx: &eframe::egui::Context,
            _frame: &mut eframe::Frame,
        ) -> Option<Box<dyn UpdatableState>> {

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


        None
    }
}
