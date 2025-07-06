use egui::Context;

pub struct Gui {
    context: Context,
}

impl Gui {
}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            context: Context::default()
        }
    }
}
