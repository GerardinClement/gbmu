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

            // ui.vertical_centered(|ui| {
            //     ui.horizontal(|ui| {
            //         ui.label("Chemin: ");
            //         ui.text_edit_singleline(&mut self.path);
            //         self.load_files();
            //     });

            //     ui.separator();

            //     egui::ScrollArea::vertical()
            //         .max_height(400.0)
            //         .show(ui, |ui| {
            //             if ui.button("..").clicked() {
            //                 self.path.pop();
            //                 while let Some(c) = self.path.pop() {
            //                     if c == '/' {
            //                         self.path.push('/');
            //                         break;
            //                     }
            //                 }
            //             }

            //             for (i, file) in self.files.iter().enumerate() {
            //                 if ui
            //                     .selectable_label(self.selected_file == Some(i), file)
            //                     .clicked()
            //                 {
            //                     self.selected_file = Some(i);
            //                     let candidate = Path::new(&self.path).join(file);
            //                     self.path.push_str(file);
            //                     if candidate.is_dir() {
            //                         self.path.push('/');
            //                     }
            //                 }
            //             }
            //         });
            // });
        });
    }

    fn load_files(&mut self) {
        self.files.clear();
        self.selected_file = None;

        // if let Ok(path) = Path::new(&self.path).read_dir() {
            // for entry in path.flatten() {
            //     let entry_as_path = entry.path();
            //     let mut enterable = false;
            //     if entry_as_path.is_dir() {
            //         enterable = true;
            //     }
            //     if let Some(extension) = entry_as_path.extension()
            //         && extension == "gb" {
            //             enterable = true;
            //         }
            //     if enterable
            //         && let Some(file_name) = entry.file_name().to_str() {
            //             self.files.push(file_name.to_string());
            //         }
            // }
            // self.files.sort();
        // }
    }
}
