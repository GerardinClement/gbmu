use crate::debugger::debbuger;
use crate::gui::views::emulation_view::update_and_get_image;
use crate::gui::{AppState, DebugingDevice};

use crate::{debugger::debbuger::{get_next_instructions, get_registers, step_button, step_mode_button, watch_address}, gui::EmulationDevice};

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};

struct DebugingDataIn<'a> {
    game_texture_handle: TextureHandle,
    is_step: bool, 
    watched_address: Vec<(&'a str, &'a str)>,
    registers_8bit: [(&'static str, u8); 6],
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
            registers_8bit: [
                ("A", self.registers.0),
                ("B", self.registers.1),
                ("C", self.registers.2),
                ("D", self.registers.3),
                ("E", self.registers.4),
                ("H", self.registers.5),
            ]
        }
    }

    fn display_interface(ctx: &Context, _frame: &mut eframe::Frame, data: DebugingDataIn) -> OutState {
        let close_button_is_clicked = eframe::egui::SidePanel::right("debug_panel")
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|inner_ui| {
                        inner_ui.heading("Debug Panel");
                        let clicked = inner_ui.with_layout(
                                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                |rtl_ui| {
                                    return rtl_ui.button("✖ Close").clicked()
                                }
                            ).inner;
                        inner_ui.separator();

                        inner_ui.add_space(8.0);
                        clicked
                    }).inner
                }).inner
            }).inner;
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.image(&data.game_texture_handle);
            });
        }).inner;

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

