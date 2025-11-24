
use crate::displayable::UpdatableState;
use crate::ui_states::game_launched::{EmulatedGame, GameLaunchedState};
use eframe::egui;

#[derive(Default)]
pub struct StartingMenuState;

impl UpdatableState for StartingMenuState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn UpdatableState>> {
        use std::cell::RefCell;

        // Use RefCell to mutate data inside the closure
        let data = RefCell::new(None);

        egui::CentralPanel::default().show(ctx, |ui| {
            let button = ui.button("Launch Game");
            if button.clicked() {
                    *data.borrow_mut() = Some(Box::new(GameLaunchedState {
                        emulated_game: EmulatedGame::new(
                            "roms/gb-test-roms/cpu_instrs/individual/02-interrupts.gb".to_string()
                        ),
                        actual_image: vec![0; 160 * 144 * 4],
                }) as Box<dyn UpdatableState>)
            }
        });

        data.into_inner()
    }
}

