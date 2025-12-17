use crate::debugger::debbuger;
use crate::gui::common::display_game;
use crate::gui::views::emulation_view::update_and_get_image;
use crate::gui::{AppState, DebugingDevice};

use crate::{debugger::debbuger::{get_next_instructions,  watch_address}, gui::EmulationDevice};

use eframe::egui::{ColorImage, Color32, Ui, Context, Grid, RichText, TextureHandle, TextureOptions};

struct DebugingDataIn<'a> {
    game_texture_handle: TextureHandle,
    is_step: bool, 
    watched_address: Vec<(&'a str, &'a str)>,
    registers: &'a (u8, u8, u8, u8, u8, u8, u8, u16, u16),
}


enum OutState {
    Emulating,
    Debuging,
}

impl DebugingDevice {
    fn update_and_get_debuging_data(&mut self, ctx: &Context) -> DebugingDataIn {
        let color_image = update_and_get_image(&mut self.core_game);
        let game_texture_handle = ctx.load_texture("gb_frame", color_image, TextureOptions::default());
        debbuger::update_info_struct(self);
        DebugingDataIn {
            is_step: self.is_step,
            game_texture_handle,
            watched_address: vec![], //TODO -> aller chercher les watched_address
            registers: &self.registers,
        }
    }

    fn display_interface(ctx: &Context, _frame: &mut eframe::Frame, data: DebugingDataIn) -> OutState {
        let close_button_is_clicked = eframe::egui::SidePanel::right("debug_panel")
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                    let close_button_clicked = ui.horizontal(|inner_ui| {
                        inner_ui.heading("Debug Panel");
                        inner_ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |rtl_ui| {
                                return rtl_ui.button("✖ Close").clicked()
                            }).inner
                    }).inner;
                    ui.separator();

                    ui.add_space(8.0);

                    let (step_mode_button_clicked, step_button_clicked) = ui.group(|mut inner_ui| {
                        inner_ui.label(RichText::new("Step Control").strong());

                        let mode_clicked = step_mode_button(&mut inner_ui, data.is_step);
                        let step_clicked = step_button(&mut inner_ui);
                        (mode_clicked, step_clicked)
                    }).inner;

                    ui.add_space(8.0);

                    ui.group(|mut inner_ui|{
                        inner_ui.label(RichText::new("Registers").strong());
                        get_registers(&mut inner_ui, &data);
                    });

                    close_button_clicked
                }).inner
            }).inner;

        display_game(data.game_texture_handle, ctx);

        if close_button_is_clicked {
            OutState::Emulating
        } else {
            OutState::Debuging
        }
    }

    pub fn debug_view(mut self, ctx: &Context, _frame: &mut eframe::Frame) -> AppState {
        let debuging_data_in = self.update_and_get_debuging_data(ctx);
        let next_state = DebugingDevice::display_interface(ctx, _frame, debuging_data_in);
        self.switch_state(next_state)
    }

    fn switch_state(self, next_state: OutState) -> AppState {
        match next_state {
            OutState::Debuging => {
                AppState::DebugingHub(self)
            }
            OutState::Emulating => {
                AppState::EmulationHub(self.into())
            }
        }
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

pub fn get_registers(ui: &mut Ui, debuging_data: &DebugingDataIn) -> bool {
    // Button to refresh registers
    let refresh_button_is_clicked = ui.horizontal(|ui| {
        ui.button("🔄 Refresh Registers").clicked()
    }).inner;

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
                ui.label(
                    RichText::new(*name).color(Color32::from_rgb(100, 200, 255)),
                );

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
                ui.label(
                    RichText::new(*name).color(Color32::from_rgb(255, 200, 100)),
                );

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
