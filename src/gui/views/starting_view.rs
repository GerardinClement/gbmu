use eframe::egui::CentralPanel;

use crate::gui::{AppState, SelectionDevice, StartingHubDevice};




impl StartingHubDevice {
    pub fn starting_view(self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) -> AppState {
        CentralPanel::default().show(ctx, |ui| {
            let button = ui.button("Start");
            if button.clicked() {
                AppState::SelectionHub(SelectionDevice::default())
            } else {
                AppState::StartingHub(StartingHubDevice{})
            }
        }).inner
    }
}


