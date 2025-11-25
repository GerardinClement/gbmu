use eframe::egui;

pub trait UpdatableState {
    fn display_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<NextState>;
    fn update(
        self: Box<Self>,
        next_state: NextState,
    ) -> Option<Box<dyn UpdatableState>>;
}


pub enum NextState {
    Debug,
    GameLaunched,
    
}
