use eframe::egui::{load::SizedTexture};

pub fn display_game(texture: SizedTexture, ui: &mut egui::Ui,) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.image(texture);
        });
    });
}

