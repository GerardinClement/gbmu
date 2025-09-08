use crate::gui::{AppState, EmulationDevice, SelectionDevice};



impl SelectionDevice {
    pub fn selection_view(
        self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame
    ) -> AppState {
        AppState::EmulationHub(
            EmulationDevice::default()
        )
    }
}


