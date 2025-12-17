use eframe::egui::{CentralPanel, Context, TextureHandle};

pub fn display_game(texture: TextureHandle, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.image(&texture);
        });
    });
}
