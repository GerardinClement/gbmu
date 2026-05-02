use eframe::egui::{load::SizedTexture, CentralPanel, Context};

pub fn display_game(texture: SizedTexture, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.image(texture);
        });
    });
}

