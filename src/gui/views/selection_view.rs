use crate::gui::{AppState, SelectionDevice};

use eframe::egui;
use std::path::Path;

enum OutState {
    Emulation,
    Selection,
}

impl SelectionDevice {
    pub fn selection_view(
        mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) -> AppState {
        self.display(ui, _frame);
        let next_state = self.next_state();
        self.update_view(next_state)
    }

    fn next_state(&mut self) -> OutState {
        let path = Path::new(&self.path);
        
        if path.is_file() {
            OutState::Emulation
        } else {
            OutState::Selection
        }
    }

    fn update_view(self, state: OutState) -> AppState {
        match state {
            OutState::Emulation => AppState::EmulationHub(self.into()),
            OutState::Selection => AppState::SelectionHub(self),
        }
    }

    fn display(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {

            if ui.button("Pick file").clicked() {
                self.file_dialog.pick_file();
            }
            ui.label(format!("Picked file: {:?}", self.picked_file));

            // Update the dialog
            
            self.file_dialog.update(ui.ctx());
            if let Some(path) = self.file_dialog.take_picked() {
                self.path = path.into_os_string().into_string().unwrap();
            }

        });
    }
}
