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
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) -> AppState {
        self.display(ctx, _frame);
        let next_state = self.next_state();
        self.update_view(next_state)
    }

    fn next_state(&mut self) -> OutState {
        let path = Path::new(&self.path[..&self.path.len() - 1]);
        if path.is_dir() {
            println!("Selection provided");
            OutState::Selection
        } else {
            self.path.pop(); // pop the / which is append each time 
            println!("Emulation launched with {}", self.path);
            OutState::Emulation
        }
    }

    fn update_view(self, state: OutState) -> AppState {
        match state {
            OutState::Emulation => AppState::EmulationHub(self.into()),
            OutState::Selection => AppState::SelectionHub(self),
        }
    }

    fn display(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Chemin: ");
                    ui.text_edit_singleline(&mut self.path);
                    self.load_files();
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        if ui.button("..").clicked() {
                            self.path.pop();
                            while let Some(c) = self.path.pop() {
                                if c == '/' {
                                    self.path.push('/');
                                    break;
                                }
                            }
                        }

                        for (i, file) in self.files.iter().enumerate() {
                            if ui
                                .selectable_label(self.selected_file == Some(i), file)
                                .clicked()
                            {
                                self.selected_file = Some(i);
                                self.path.push_str(file);
                                self.path.push('/');
                            }
                        }
                    });
            });
        });
    }

    fn load_files(&mut self) {
        self.files.clear();
        self.selected_file = None;

        if let Ok(path) = Path::new(&self.path).read_dir() {
            for entry in path.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    self.files.push(file_name.to_string());
                }
            }
            self.files.sort();
        }
    }
}
