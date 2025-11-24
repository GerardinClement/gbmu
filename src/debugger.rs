#![allow(unused_variables)]
#![allow(dead_code)]

pub mod debbuger {

    use crate::{DebugCommandQueries, DebugResponse};
    use crate::ui_states::debuging_game::DebugedGame;
    use eframe::egui;

    pub fn update_info_struct(game: &mut DebugedGame) {
        if let Ok(debug) = game.debug_response_receiver.try_recv() {
            match debug {
                DebugResponse::AddressesWatched(wa) => {
                    game.watched_adress = wa;
                }
                DebugResponse::StepModeSet(value) => {
                    game.is_step = value;
                }
                DebugResponse::NextInstructions(list) => {
                    game.next_instructions.clear();
                    list.iter().for_each(|f| game.next_instructions.push(*f));
                }
                DebugResponse::InstructionsExecuted(s) => {
                    todo!();
                }
                DebugResponse::Registers(a, b, c, d, e, h, l, hl, sp) => {
                    game.registers = (a, b, c, d, e, h, l, hl, sp);
                }
            }
        }
    }

    pub fn step_mode_button(ui: &mut egui::Ui, game: &mut DebugedGame) {
        let s = if game.is_step {
            "Desactivate step mode".to_string()
        } else {
            "Activate step mode".to_string()
        };
        let button = ui.button(s);
        if button.clicked() {
            game.set_step_mode();
        }
    }

    pub fn step_button(ui: &mut egui::Ui, game: &mut DebugedGame) {
        let button = ui.button("Next Step");
        if button.clicked() {
            game.executed_next_step(1);
        }
    }

    pub fn get_registers(ui: &mut egui::Ui, game: &mut DebugedGame) {
        // Button to refresh registers
        ui.add_space(8.0);

        // Display registers in a structured table format
        egui::Grid::new("registers_grid")
            .num_columns(4)
            .spacing([20.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                // Headers
                ui.label(egui::RichText::new("Reg").strong());
                ui.label(egui::RichText::new("Hex").strong());
                ui.label(egui::RichText::new("Dec").strong());
                ui.label(egui::RichText::new("Binary").strong());
                ui.end_row();

                // 8-bit registers
                let registers_8bit = [
                    ("A", game.registers.0),
                    ("B", game.registers.1),
                    ("C", game.registers.2),
                    ("D", game.registers.3),
                    ("E", game.registers.4),
                    ("H", game.registers.5),
                ];

                for (name, value) in registers_8bit.iter() {
                    ui.label(
                        egui::RichText::new(*name).color(egui::Color32::from_rgb(100, 200, 255)),
                    );

                    ui.label(egui::RichText::new(format!("0x{:02X}", value)).monospace());

                    ui.label(
                        egui::RichText::new(format!("{:3}", value))
                            .monospace()
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );

                    ui.label(
                        egui::RichText::new(format!("{:08b}", value))
                            .monospace()
                            .color(egui::Color32::from_rgb(100, 255, 100)),
                    );

                    ui.end_row();
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();
                ui.end_row();

                // 16-bit registers
                let registers_16bit = [
                    ("L", game.registers.6 as u16),
                    ("HL", game.registers.7),
                    ("SP", game.registers.8),
                ];

                for (name, value) in registers_16bit.iter() {
                    ui.label(
                        egui::RichText::new(*name).color(egui::Color32::from_rgb(255, 200, 100)),
                    );

                    ui.label(egui::RichText::new(format!("0x{:04X}", value)).monospace());

                    ui.label(
                        egui::RichText::new(format!("{:5}", value))
                            .monospace()
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );

                    ui.label(
                        egui::RichText::new(format!("{:016b}", value))
                            .monospace()
                            .color(egui::Color32::from_rgb(100, 255, 100)),
                    );

                    ui.end_row();
                }
            });
    }

    pub fn get_next_instructions(ui: &mut egui::Ui, game: &mut DebugedGame) {
        // Input section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Instructions to fetch:");

                // Decimal drag value
                ui.add(
                    egui::DragValue::new(&mut game.nb_instruction)
                        .speed(1.0)
                        .clamp_range(0..=255)
                        .prefix("Dec: "),
                );

                // Fetch button
                let fetch_btn = ui.add_sized([100.0, 20.0], egui::Button::new("📋 Fetch"));

                if fetch_btn.clicked() {
                    game.get_next_instructions(game.nb_instruction);
                }
            });
        });

        ui.add_space(8.0);

        // Display instructions
        if game.nb_instruction > 0 && !game.next_instructions.is_empty() {
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Next Instructions").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("({})", game.next_instructions.len()));
                });
            });

            ui.add_space(4.0);

            // Headers (outside scroll area, always visible)
            egui::Grid::new("instructions_header")
                .num_columns(4)
                .spacing([15.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("#").strong());
                    ui.label(egui::RichText::new("Hex").strong());
                    ui.label(egui::RichText::new("Dec").strong());
                    ui.label(egui::RichText::new("Binary").strong());
                });

            ui.separator();

            // Scrollable content area with fixed height
            ui.push_id("instruction_scoll", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .auto_shrink([true; 2])
                    .show(ui, |ui| {
                        egui::Grid::new("instructions_grid")
                            .num_columns(4)
                            .spacing([15.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                // Instructions
                                for (i, instruction) in game.next_instructions.iter().enumerate() {
                                    // Index
                                    ui.label(
                                        egui::RichText::new(format!("{}", i + 1))
                                            .color(egui::Color32::from_rgb(150, 150, 150)),
                                    );

                                    // Hex value
                                    ui.label(
                                        egui::RichText::new(format!("0x{:02X}", instruction))
                                            .monospace()
                                            .color(egui::Color32::from_rgb(100, 200, 255)),
                                    );

                                    // Decimal value
                                    ui.label(
                                        egui::RichText::new(format!("{:3}", instruction))
                                            .monospace()
                                            .color(egui::Color32::from_rgb(150, 150, 150)),
                                    );

                                    // Binary value
                                    ui.label(
                                        egui::RichText::new(format!("{:08b}", instruction))
                                            .monospace()
                                            .color(egui::Color32::from_rgb(100, 255, 100)),
                                    );

                                    ui.end_row();
                                }
                            });
                    });
            });
        } else if game.nb_instruction > 0 && game.next_instructions.is_empty() {
            ui.label(
                egui::RichText::new("No instructions fetched yet. Click 'Fetch' to load.")
                    .italics()
                    .color(egui::Color32::DARK_GRAY),
            );
        }
    }

    pub fn watch_address(ui: &mut egui::Ui, game: &mut DebugedGame) {
        // Input section with better layout
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Address:");

                // Hex input with better formatting
                ui.label("(0x)");
                let response_changed = ui
                    .add(
                        egui::TextEdit::singleline(&mut game.hex_string)
                            .desired_width(60.0)
                            .hint_text("0000")
                            .char_limit(4),
                    )
                    .changed();

                // Parse hex input
                if let Ok(value) = u16::from_str_radix(game.hex_string.as_ref(), 16) {
                    if response_changed {
                        game.watched_address_value = value;
                    }
                }

                // Alternative: decimal slider
                ui.add(
                    egui::DragValue::new(&mut game.watched_address_value)
                        .speed(1.0)
                        .clamp_range(0x0000..=0xFFFF)
                        .prefix("Dec: "),
                );

                // Watch button with better styling
                let watch_btn = ui.add_sized([80.0, 20.0], egui::Button::new("📌 Watch"));

                if watch_btn.clicked() {
                    let is_already_watched = game
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .any(|(address, _)| *address == game.watched_address_value);

                    if !is_already_watched {
                        game.watch_address(game.watched_address_value);
                        game.error_message = None;
                    } else {
                        game.error_message = Some(format!(
                            "Address 0x{:04X} is already being watched",
                            game.watched_address_value
                        ));
                    }
                }
            });
        });

        // Error message display
        if let Some(ref error_msg) = game.error_message {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").color(egui::Color32::YELLOW));
                ui.colored_label(egui::Color32::YELLOW, error_msg);
            });
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Watched addresses section with header
        ui.horizontal(|ui| {
            ui.heading("Watched Addresses");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !game.watched_adress.addresses_n_values.is_empty() {
                    ui.label(format!(
                        "({})",
                        game.watched_adress.addresses_n_values.len()
                    ));
                }
            });
        });

        ui.add_space(4.0);

        // Display watched addresses with better formatting
        if game.watched_adress.addresses_n_values.is_empty() {
            ui.label(
                egui::RichText::new("No addresses being watched")
                    .italics()
                    .color(egui::Color32::DARK_GRAY),
            );
        } else {
            let mut address_to_remove = None;
            ui.push_id("watched_address", |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([true; 2])
                    .max_height(300.0)
                    .show(ui, |ui| {
                        // Table header
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("#").strong());
                            ui.label(egui::RichText::new("Address").strong());
                            ui.label(egui::RichText::new("Value (Hex)").strong());
                            ui.label(egui::RichText::new("Value (Dec)").strong());
                            ui.label(egui::RichText::new("Binary").strong());
                        });

                        ui.separator();

                        for (i, (address, value)) in
                            game.watched_adress.addresses_n_values.iter().enumerate()
                        {
                            ui.horizontal(|ui| {
                                // Index
                                ui.label(format!("{}", i + 1));

                                // Address in hex
                                ui.label(
                                    egui::RichText::new(format!("0x{:04X}", address))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(100, 200, 255)),
                                );

                                // Value in hex
                                ui.label(
                                    egui::RichText::new(format!("0x{:02X}", value)).monospace(),
                                );

                                // Value in decimal
                                ui.label(
                                    egui::RichText::new(format!("{:3}", value))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(150, 150, 150)),
                                );

                                // Value in binary
                                ui.label(
                                    egui::RichText::new(format!("{:08b}", value))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(100, 255, 100)),
                                );

                                // Spacer to push remove button to the right
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
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
                            if i < game.watched_adress.addresses_n_values.len() - 1 {
                                ui.add_space(2.0);
                            }
                        }
                    });

                if let Some(addr) = address_to_remove
                    && let Some(index) = game
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .position(|(address, _)| *address == addr)
                {
                    game.watched_adress.addresses_n_values.remove(index);
                    game.watch_address(address_to_remove.unwrap());
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
                        game.watched_address_value = *addr;
                        if !game
                            .watched_adress
                            .addresses_n_values
                            .iter()
                            .any(|(address, _)| *address == *addr)
                        {
                            game.watch_address(*addr);
                        }
                    }
                }
            });
        });
    }

    impl DebugedGame {
        pub fn execute_instruction(&self, instr: u8) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::ExecuteInstruction(instr));
        }

        pub fn get_next_instructions(&self, instr_nb: u8) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::GetNextInstructions(instr_nb));
        }

        pub fn get_registers(&self) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::GetRegisters);
        }

        pub fn set_step_mode(&mut self) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::SetStepMode);
        }

        pub fn set_debug_mode(&self) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::SetDebugMode);
        }

        pub fn executed_next_step(&self, nb_instru: usize) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::ExecuteNextInstructions(nb_instru));
        }

        pub fn watch_address(&self, address: u16) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::WatchAddress(address));
        }

        pub fn get_watched_addresses(&self) {
            let _ = self
                .debug_sender
                .try_send(DebugCommandQueries::GetAddresses);
        }
    }
}
