use eframe::egui;

pub trait UpdatableState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn UpdatableState>>;
}
