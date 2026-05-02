use eframe::egui::{CentralPanel, Direction, Layout};
use crate::gui::{AppState, SelectionDevice, StartingHubDevice};
use crate::gui::themes::dark_theme::get_dark_theme_visual;


impl StartingHubDevice {
    pub fn starting_view(
        self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) -> AppState {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ctx.set_visuals(get_dark_theme_visual());
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    if ui.button("Start").clicked() {
                        AppState::SelectionHub(SelectionDevice::default())
                    } else {
                        AppState::StartingHub(StartingHubDevice {})
                    }
                })
                .inner
            })
            .inner
    }
}
