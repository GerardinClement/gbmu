use crate::gui::common::display_game;

use eframe::egui::{
    Align, Button, Color32, Context, DragValue, Grid, Layout, RichText, ScrollArea, SidePanel,
    TextEdit, Ui,
};

use super::{DebugingDataIn, DebugingDataOut};

pub fn display_interface(
    ctx: &Context,
    _frame: &mut eframe::Frame,
    data: DebugingDataIn,
) -> DebugingDataOut {
    let (
        close_btn_clicked,
        step_mode_btn_clkd,
        stp_btn_clkd,
        refresh_register_clicked,
        instructions_are_requested,
        nb_instruction_requested,
        hex_string,
        register_new_addr,
    ): (bool, bool, bool, bool, bool, u8, String, bool) = SidePanel::right("debug_panel")
        .resizable(true)
        .default_width(400.0)
        .min_width(300.0)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .show(ui, |ui| {
                    let close_button_is_clicked: bool = ui
                        .horizontal(|inner_ui| {
                            inner_ui.heading("Debug Panel");
                            inner_ui
                                .with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
                                    rtl_ui.button("✖ Close").clicked()
                                })
                                .inner
                        })
                        .inner;
                    ui.separator();

                    ui.add_space(8.0);

                    let (step_mode_button_clicked, step_button_clicked): (bool, bool) = ui
                        .group(|inner_ui| {
                            inner_ui.label(RichText::new("Step Control").strong());

                            let mode_clicked = step_mode_button(inner_ui, data.is_step);
                            let step_clicked = step_button(inner_ui);
                            (mode_clicked, step_clicked)
                        })
                        .inner;

                    ui.add_space(8.0);

                    let refresh_register_clicked: bool = ui
                        .group(|inner_ui| {
                            inner_ui.label(RichText::new("Registers").strong());
                            get_registers(inner_ui, &data)
                        })
                        .inner;

                    let (nb_instruction_requested, instructions_are_requested): (u8, bool) = ui
                        .group(|inner_ui| {
                            inner_ui.label(RichText::new("Next Instructions").strong());
                            get_next_instructions(inner_ui, &data)
                        })
                        .inner;

                    ui.add_space(8.0);

                    let (hex_string, register_new_addr) = ui
                        .group(|inner_ui| {
                            inner_ui.label(RichText::new("Memory Watch").strong());
                            watch_address(inner_ui, &data)
                        })
                        .inner;

                    (
                        close_button_is_clicked,
                        step_mode_button_clicked,
                        step_button_clicked,
                        refresh_register_clicked,
                        instructions_are_requested,
                        nb_instruction_requested,
                        hex_string,
                        register_new_addr,
                    )
                })
                .inner
        })
        .inner;

    if let Some(sized_texture) = data.sized_texture {
        display_game(sized_texture, ctx);
    }

    DebugingDataOut {
        step_clicked: stp_btn_clkd,
        step_mode_clicked: step_mode_btn_clkd,
        close_btn_clicked,
        refresh_register_clicked,
        instructions_are_requested,
        nb_instruction_requested,
        hex_string,
        register_new_addr,
    }
}

fn step_mode_button(ui: &mut Ui, is_in_step_mode: bool) -> bool {
    let s = if is_in_step_mode {
        "Desactivate step mode".to_string()
    } else {
        "Activate step mode".to_string()
    };
    ui.button(s).clicked()
}

fn step_button(ui: &mut Ui) -> bool {
    ui.button("Next Step").clicked()
}

fn get_registers(ui: &mut Ui, debuging_data: &DebugingDataIn) -> bool {
    // Button to refresh registers
    let refresh_button_is_clicked = ui
        .horizontal(|ui| ui.button("🔄 Refresh Registers").clicked())
        .inner;

    ui.add_space(8.0);

    // Display registers in a structured table format
    Grid::new("registers_grid")
        .num_columns(4)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            // Headers
            ui.label(RichText::new("Reg").strong());
            ui.label(RichText::new("Hex").strong());
            ui.label(RichText::new("Dec").strong());
            ui.label(RichText::new("Binary").strong());
            ui.end_row();

            let registers_8bit = [
                ("A", debuging_data.registers.0),
                ("B", debuging_data.registers.1),
                ("C", debuging_data.registers.2),
                ("D", debuging_data.registers.3),
                ("E", debuging_data.registers.4),
                ("H", debuging_data.registers.5),
            ];

            for (name, value) in registers_8bit.iter() {
                ui.label(RichText::new(*name).color(Color32::from_rgb(100, 200, 255)));

                ui.label(RichText::new(format!("0x{:02X}", value)).monospace());

                ui.label(
                    RichText::new(format!("{:3}", value))
                        .monospace()
                        .color(Color32::from_rgb(150, 150, 150)),
                );

                ui.label(
                    RichText::new(format!("{:08b}", value))
                        .monospace()
                        .color(Color32::from_rgb(100, 255, 100)),
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
                ("L", debuging_data.registers.6 as u16),
                ("HL", debuging_data.registers.7),
                ("SP", debuging_data.registers.8),
            ];

            for (name, value) in registers_16bit.iter() {
                ui.label(RichText::new(*name).color(Color32::from_rgb(255, 200, 100)));

                ui.label(RichText::new(format!("0x{:04X}", value)).monospace());

                ui.label(
                    RichText::new(format!("{:5}", value))
                        .monospace()
                        .color(Color32::from_rgb(150, 150, 150)),
                );

                ui.label(
                    RichText::new(format!("{:016b}", value))
                        .monospace()
                        .color(Color32::from_rgb(100, 255, 100)),
                );

                ui.end_row();
            }
        });
    refresh_button_is_clicked
}

fn get_next_instructions(ui: &mut Ui, data: &DebugingDataIn) -> (u8, bool) {
    // Input section
    let instruction_requested_tuple = ui
        .group(|ui| {
            ui.horizontal(|ui| {
                let mut nb_instructions = data.nb_instruction;
                ui.label("Instructions to fetch:");

                // Decimal drag value
                ui.add(
                    DragValue::new(&mut nb_instructions)
                        .speed(1.0)
                        .clamp_range(0..=255)
                        .prefix("Dec: "),
                );

                // Fetch button
                let fetch_btn = ui.add_sized([100.0, 20.0], Button::new("📋 Fetch"));
                (nb_instructions, fetch_btn.clicked())
            })
            .inner
        })
        .inner;

    ui.add_space(8.0);

    // Display instructions
    if data.nb_instruction > 0 && !data.next_instructions.is_empty() {
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Next Instructions").strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("({})", data.next_instructions.len()));
            });
        });

        ui.add_space(4.0);

        // Headers (outside scroll area, always visible)
        Grid::new("instructions_header")
            .num_columns(4)
            .spacing([15.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("#").strong());
                ui.label(RichText::new("Hex").strong());
                ui.label(RichText::new("Dec").strong());
                ui.label(RichText::new("Binary").strong());
            });

        ui.separator();

        // Scrollable content area with fixed height
        ui.push_id("instruction_scoll", |ui| {
            ScrollArea::vertical()
                .max_height(100.0)
                .auto_shrink([true; 2])
                .show(ui, |ui| {
                    Grid::new("instructions_grid")
                        .num_columns(4)
                        .spacing([15.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Instructions
                            for (i, instruction) in data.next_instructions.iter().enumerate() {
                                // Index
                                ui.label(
                                    RichText::new(format!("{}", i + 1))
                                        .color(Color32::from_rgb(150, 150, 150)),
                                );

                                // Hex value
                                ui.label(
                                    RichText::new(format!("0x{:02X}", instruction))
                                        .monospace()
                                        .color(Color32::from_rgb(100, 200, 255)),
                                );

                                // Decimal value
                                ui.label(
                                    RichText::new(format!("{:3}", instruction))
                                        .monospace()
                                        .color(Color32::from_rgb(150, 150, 150)),
                                );

                                // Binary value
                                ui.label(
                                    RichText::new(format!("{:08b}", instruction))
                                        .monospace()
                                        .color(Color32::from_rgb(100, 255, 100)),
                                );

                                ui.end_row();
                            }
                        });
                });
        });
    } else if data.nb_instruction > 0 && data.next_instructions.is_empty() {
        ui.label(
            RichText::new("No instructions fetched yet. Click 'Fetch' to load.")
                .italics()
                .color(Color32::DARK_GRAY),
        );
    };
    instruction_requested_tuple
}

fn watch_address(ui: &mut Ui, data: &DebugingDataIn) -> (String, bool) {
    let mut hex_string = data.hex_string.clone();
    // Input section with better layout
    let register_new_addr: bool = ui
        .group(|inner_ui| {
            inner_ui
                .horizontal(|h_ui| {
                    h_ui.label("Address:");

                    // Hex input with better formatting
                    h_ui.label("(0x)");
                    let response_changed = h_ui
                        .add(
                            TextEdit::singleline(&mut hex_string)
                                .desired_width(60.0)
                                .hint_text("0000")
                                .char_limit(4),
                        )
                        .changed();

                    let watch_btn = h_ui.add_sized([80.0, 20.0], Button::new("📌 Watch"));

                    watch_btn.clicked()
                })
                .inner
        })
        .inner;

    // Error message display
    if let Some(error_msg) = data.error_message {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚠").color(Color32::YELLOW));
            ui.colored_label(Color32::YELLOW, error_msg);
        });
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Watched addresses section with header
    ui.horizontal(|ui| {
        ui.heading("Watched Addresses");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !data.watched_address.addresses_n_values.is_empty() {
                ui.label(format!(
                    "({})",
                    data.watched_address.addresses_n_values.len()
                ));
            }
        });
    });

    ui.add_space(4.0);

    // Display watched addresses with better formatting
    if data.watched_address.addresses_n_values.is_empty() {
        ui.label(
            RichText::new("No addresses being watched")
                .italics()
                .color(Color32::DARK_GRAY),
        );
    } else {
        let mut address_to_remove = None;
        ui.push_id("watched_address", |ui| {
            ScrollArea::vertical()
                .auto_shrink([true; 2])
                .max_height(300.0)
                .show(ui, |ui| {
                    // Table header
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("#").strong());
                        ui.label(RichText::new("Address").strong());
                        ui.label(RichText::new("Value (Hex)").strong());
                        ui.label(RichText::new("Value (Dec)").strong());
                        ui.label(RichText::new("Binary").strong());
                    });

                    ui.separator();

                    for (i, (address, value)) in
                        data.watched_address.addresses_n_values.iter().enumerate()
                    {
                        ui.horizontal(|ui| {
                            // Index
                            ui.label(format!("{}", i + 1));

                            // Address in hex
                            ui.label(
                                RichText::new(format!("0x{:04X}", address))
                                    .monospace()
                                    .color(Color32::from_rgb(100, 200, 255)),
                            );

                            // Value in hex
                            ui.label(RichText::new(format!("0x{:02X}", value)).monospace());

                            // Value in decimal
                            ui.label(
                                RichText::new(format!("{:3}", value))
                                    .monospace()
                                    .color(Color32::from_rgb(150, 150, 150)),
                            );

                            // Value in binary
                            ui.label(
                                RichText::new(format!("{:08b}", value))
                                    .monospace()
                                    .color(Color32::from_rgb(100, 255, 100)),
                            );

                            // Spacer to push remove button to the right
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Remove button
                                if ui.small_button("✖").on_hover_text("Remove").clicked() {
                                    address_to_remove = Some(*address);
                                }
                            });
                        });

                        // Subtle separator between entries
                        if i < data.watched_address.addresses_n_values.len() - 1 {
                            ui.add_space(2.0);
                        }
                    }
                });
            /*

                        if let Some(addr) = address_to_remove
                        && let Some(index) = data
                            .watched_address
                            .addresses_n_values
                            .iter()
                            .position(|(address, _)| *address == addr)
                        {
                            data.watched_address.addresses_n_values.remove(index);
                            data.watch_address(address_to_remove.unwrap());
                        }
            */
        });
    }
    ui.add_space(4.0);
    (hex_string, register_new_addr)

    /*
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
                    data.watched_address_value = *addr;
                    if !data
                        .watched_address
                        .addresses_n_values
                        .iter()
                        .any(|(address, _)| *address == *addr)
                    {
                        data.watch_address(*addr);
                    }
                }
            }
        });
    });
    */
}
