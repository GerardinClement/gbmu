use crate::gui::{AppState, DebugingDevice};



impl DebugingDevice {
    pub fn debug_view(self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) -> AppState {
        eframe::egui::SidePanel::right("debug_panel")
            .default_width(400.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                    /*
                    if let Some(state) = ui.horizontal(|ui| {
                        ui.heading("Debug Panel");
                        ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                            |ui| {
                                if ui.button("✖ Close").clicked() {
                                    Some(AppState::EmulationHub(EmulationDevice::from(self)))
                                } else {
                                    None
                                }
                            },
                        ).inner
                    }).inner {
                        return Some(state)
                    }
                    ui.separator();

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(eframe::egui::RichText::new("Step Control").strong());
                        step_mode_button(ui, &mut self);
                        step_button(ui, &self);
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(eframe::egui::RichText::new("Registers").strong());
                        get_registers(ui, &self);
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(eframe::egui::RichText::new("Next Instructions").strong());
                        get_next_instructions(ui, &mut self);
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(eframe::egui::RichText::new("Memory Watch").strong());
                        watch_address(ui, &mut self);
                    });
                */
                }).inner
            }).inner;


        AppState::DebugingHub(DebugingDevice::from(self))
    }
}

