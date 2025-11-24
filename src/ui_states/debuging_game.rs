use tokio::task::JoinHandle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::debugger::debbuger::{watch_address, get_next_instructions, get_registers, step_button, step_mode_button};
use crate::displayable::UpdatableState;
use crate::ui_states::starting_menu::StartingMenuState;


pub struct DebugingGame {
    pub emulated_game: DebugedGame,
    pub actual_image: Vec<u8>,
}

pub struct DebugedGame {
    handler: JoinHandle<()>,
    input_sender: Sender<Vec<u8>>,
    image_receiver: Receiver<Vec<u8>>,
    pub debug_sender: Sender<DebugCommandQueries>,
    pub debug_receiver: Receiver<DebugResponse>,

    pub next_instructions: Vec<u16>,
    pub watched_adress: WatchedAdresses,
    pub registers: (u8, u8, u8, u8, u8, u8, u8, u16, u16),
    pub is_debug: bool,
    pub is_step: bool,
    pub watched_address_value: u16,
    pub nb_instruction: u8,
    pub error_message: Option<String>,
    pub hex_string: String,
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
    pub addresses_n_values: Vec<(u16, u16)>,
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

        use std::cell::RefCell;
        let data = RefCell::new(None);
        eframe::egui::SidePanel::right("debug_panel")
                    .resizable(true)
                    .default_width(400.0)
                    .min_width(300.0)
                    .show(ctx, |ui| {
                        eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.separator();

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(eframe::egui::RichText::new("Step Control").strong());
                                step_mode_button(ui, game);
                                step_button(ui, game);
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(eframe::egui::RichText::new("Registers").strong());
                                ui.horizontal(|ui| {
                                    if ui.button("🔄 Refresh Registers").clicked() {
                                        self.emulated_game.get_registers();
                                    }
                                });
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Instructions to fetch:");

                                        // Decimal drag value
                                        ui.add(
                                            eframe::egui::DragValue::new(&mut self.emulated_game.nb_instruction)
                                                .speed(1.0)
                                                .clamp_range(0..=255)
                                                .prefix("Dec: "),
                                        );

                                        // Fetch button
                                        let fetch_btn = ui.add_sized([100.0, 20.0], eframe::egui::Button::new("📋 Fetch"));

                                        if fetch_btn.clicked() {
                                            self.emulated_game.get_next_instructions(self.emulated_game.nb_instruction);
                                        }
                                    });
                                });
                            });

                            ui.add_space(8.0);

                            ui.group(|ui| {
                                ui.label(eframe::egui::RichText::new("Memory Watch").strong());




        // Input section with better layout
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Address:");

                // Hex input with better formatting
                ui.label("(0x)");
                let response_changed = ui
                    .add(
                        eframe::egui::TextEdit::singleline(&mut self.emulated_game.hex_string)
                            .desired_width(60.0)
                            .hint_text("0000")
                            .char_limit(4),
                    )
                    .changed();

                // Parse hex input
                if let Ok(value) = u16::from_str_radix(self.emulated_game.hex_string.as_ref(), 16) {
                    if response_changed {
                        self.emulated_game.watched_address_value = value;
                    }
                }

                // Alternative: decimal slider
                ui.add(
                    eframe::egui::DragValue::new(&mut self.emulated_game.watched_address_value)
                        .speed(1.0)
                        .clamp_range(0x0000..=0xFFFF)
                        .prefix("Dec: "),
                );

                // Watch button with better styling
                let watch_btn = ui.add_sized([80.0, 20.0], eframe::egui::Button::new("📌 Watch"));

                if watch_btn.clicked() {
                    let is_already_watched = self.emulated_game
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .any(|(address, _)| *address == self.emulated_game.watched_address_value);

                    if !is_already_watched {
                        self.emulated_game.watch_address(self.emulated_game.watched_address_value);
                        self.emulated_game.error_message = None;
                    } else {
                        self.emulated_game.error_message = Some(format!(
                            "Address 0x{:04X} is already being watched",
                            self.emulated_game.watched_address_value
                        ));
                    }
                }
            });
        });

        // Error message display
        if let Some(ref error_msg) = self.emulated_game.error_message {
            ui.horizontal(|ui| {
                ui.label(eframe::egui::RichText::new("⚠").color(eframe::egui::Color32::YELLOW));
                ui.colored_label(eframe::egui::Color32::YELLOW, error_msg);
            });
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Watched addresses section with header
        ui.horizontal(|ui| {
            ui.heading("Watched Addresses");
            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                if !self.emulated_game.watched_adress.addresses_n_values.is_empty() {
                    ui.label(format!(
                        "({})",
                        self.emulated_game.watched_adress.addresses_n_values.len()
                    ));
                }
            });
        });

        ui.add_space(4.0);

        // Display watched addresses with better formatting
        if self.emulated_game.watched_adress.addresses_n_values.is_empty() {
            ui.label(
                eframe::egui::RichText::new("No addresses being watched")
                    .italics()
                    .color(eframe::egui::Color32::DARK_GRAY),
            );
        } else {
            let mut address_to_remove = None;
            ui.push_id("watched_address", |ui| {
                eframe::egui::ScrollArea::vertical()
                    .auto_shrink([true; 2])
                    .max_height(300.0)
                    .show(ui, |ui| {
                        // Table header
                        ui.horizontal(|ui| {
                            ui.label(eframe::egui::RichText::new("#").strong());
                            ui.label(eframe::egui::RichText::new("Address").strong());
                            ui.label(eframe::egui::RichText::new("Value (Hex)").strong());
                            ui.label(eframe::egui::RichText::new("Value (Dec)").strong());
                            ui.label(eframe::egui::RichText::new("Binary").strong());
                        });

                        ui.separator();

                        for (i, (address, value)) in
                            self.emulated_game.watched_adress.addresses_n_values.iter().enumerate()
                        {
                            ui.horizontal(|ui| {
                                // Index
                                ui.label(format!("{}", i + 1));

                                // Address in hex
                                ui.label(
                                    eframe::egui::RichText::new(format!("0x{:04X}", address))
                                        .monospace()
                                        .color(eframe::egui::Color32::from_rgb(100, 200, 255)),
                                );

                                // Value in hex
                                ui.label(
                                    eframe::egui::RichText::new(format!("0x{:02X}", value)).monospace(),
                                );

                                // Value in decimal
                                ui.label(
                                    eframe::egui::RichText::new(format!("{:3}", value))
                                        .monospace()
                                        .color(eframe::egui::Color32::from_rgb(150, 150, 150)),
                                );

                                // Value in binary
                                ui.label(
                                    eframe::egui::RichText::new(format!("{:08b}", value))
                                        .monospace()
                                        .color(eframe::egui::Color32::from_rgb(100, 255, 100)),
                                );

                                // Spacer to push remove button to the right
                                ui.with_layout(
                                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                    |ui| {
                                        // Remove button
                                        if ui.small_button("✖").on_hover_text("Remove").clicked()
                                        {
                                            address_to_remove = Some(*address);
                                        }
                                    },
                                );
                            });

                            // Subtle separator between entries
                            if i < self.emulated_game.watched_adress.addresses_n_values.len() - 1 {
                                ui.add_space(2.0);
                            }
                        }
                    });

                if let Some(addr) = address_to_remove
                    && let Some(index) = self.emulated_game
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .position(|(address, _)| *address == addr)
                {
                    self.emulated_game.watched_adress.addresses_n_values.remove(index);
                    self.emulated_game.watch_address(address_to_remove.unwrap());
                }
            });
        }
        ui.add_space(4.0);

        // Optional: Quick access to common GameBoy memory regions
        ui.collapsing("Quick Add Memory Regions", |ui| {
            ui.horizontal_wrapped(|ui| {
                let regions = [
                    ("DIV ($FF04)", 0xFF04),
                    ("TIMA ($FF05)", 0xFF05),
                    ("TMA ($FF06)", 0xFF06),
                    ("TAC ($FF07)", 0xFF07),
                    ("IF ($FF0F)", 0xFF0F),
                    ("LCDC ($FF40)", 0xFF40),
                    ("STAT ($FF41)", 0xFF41),
                    ("SCY ($FF42)", 0xFF42),
                    ("SCX ($FF43)", 0xFF43),
                    ("LY ($FF44)", 0xFF44),
                    ("LYC ($FF45)", 0xFF45),
                    ("BGP ($FF47)", 0xFF47),
                    ("HMAP ($FF47)", 0xFF80),
                ];

                for (name, addr) in regions.iter() {
                    if ui.small_button(*name).clicked() {
                        self.emulated_game.watched_address_value = *addr;
                        if !self.emulated_game
                            .watched_adress
                            .addresses_n_values
                            .iter()
                            .any(|(address, _)| *address == *addr)
                        {
                            self.emulated_game.watch_address(*addr);
                        }
                    }
                }
            });
        });
















                            });
                            ui.horizontal(|ui| {
                                ui.heading("Debug Panel");
                                ui.with_layout(
                                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                    |ui| {
                                        if ui.button("✖ Close").clicked() {
                                    *data.borrow_mut() = Some(Box::new(StartingMenuState::default()) as Box<dyn UpdatableState>);

                                        }
                                    },
                                );
                            });
                        });
                    });


        None
    }
}
