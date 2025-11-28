use crate::displayable::{NextState, UpdatableState};
use crate::ui_states::game_launched::{EmulatedGame, GameLaunchedState};
use eframe::egui;

#[derive(Default)]
pub struct StartingMenuState;

impl UpdatableState for StartingMenuState {
    fn display_gui(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<NextState> {
        use std::cell::RefCell;

        // Use RefCell to mutate data inside the closure
        let data = RefCell::new(None);

        egui::CentralPanel::default().show(ctx, |ui| {
            let button = ui.button("Launch Game");
            if button.clicked() {
                *data.borrow_mut() = Some(NextState::GameLaunched)
            }
        });
        data.into_inner()
    }

    fn update(
        self: Box<Self>,
        next_state: NextState,
    ) -> Box<dyn UpdatableState>{
        println!("this is comming");
        match next_state {
            NextState::GameLaunched => {
                println!("this is comming in ");
                Box::new(
                    GameLaunchedState {
                    emulated_game: EmulatedGame::from_rom_path(
                        "gb-test-roms/cpu_instrs/individual/02-interrupts.gb".to_string(),
                    ),
                    actual_image: vec![0; 160 * 144 * 4],
                }) as Box<dyn UpdatableState>
            }
            NextState::Debug => {
                todo!()
            }
        }
    }
}
