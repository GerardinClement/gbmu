use eframe::egui::{CentralPanel, Direction, Layout};

use crate::gui::{AppState, SelectionDevice, StartingHubDevice};

impl StartingHubDevice {
    pub fn starting_view(
        self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) -> AppState {
        CentralPanel::default()
            .show(ctx, |ui| {
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
