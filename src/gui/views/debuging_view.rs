mod display;

use crate::debugger::debbuger;
use crate::gui::views::emulation_view::update_and_get_image;
use crate::gui::{AppState, DebugingDevice, WatchedAdresses};

use eframe::egui::{Context, TextureHandle, TextureOptions};

use display::display_interface;

struct DebugingDataIn<'a> {
    game_texture_handle: TextureHandle,
    is_step: bool, 
    watched_address: &'a WatchedAdresses,
    registers: &'a (u8, u8, u8, u8, u8, u8, u8, u16, u16),
    nb_instruction: u8,
    next_instructions: &'a Vec<u16>,
    hex_string: &'a String,
    error_message: Option<&'a String>
}

#[derive(Debug)]
struct DebugingDataOut {
    close_btn_clicked: bool,
    step_clicked: bool,
    step_mode_clicked: bool,
    refresh_register_clicked: bool,
    instructions_are_requested: bool, 
    nb_instruction_requested: u8,
    hex_string: String,
    register_new_addr: bool,
}


enum OutState {
    Emulating,
    Debuging,
}



impl DebugingDevice {
    fn execute_changes(&mut self, data: DebugingDataOut) -> OutState {
        if data.close_btn_clicked {
            return OutState::Emulating;
        }

        if data.step_mode_clicked {
            self.request_step_mode();
        }

        self.nb_instruction = data.nb_instruction_requested as usize;
        if data.step_clicked {
            self.executed_next_step(1);
        }

        if data.instructions_are_requested {
            self.get_next_instructions(data.nb_instruction_requested);
        }

        if data.refresh_register_clicked {
            self.request_registers();
        }

        self.hex_string = data.hex_string; 
        if let Ok(result) = u16::from_str_radix(self.hex_string.as_ref(), 16) {

        }
        OutState::Debuging
    }

    pub fn debug_view(mut self, ctx: &Context, _frame: &mut eframe::Frame) -> AppState {
        let debuging_data_in = self.update_and_get_debuging_data(ctx);
        let actions_to_perform = display_interface(ctx, _frame, debuging_data_in);
        println!("{actions_to_perform:?}");
        let next_state = self.execute_changes(actions_to_perform);
        self.switch_state(next_state)
    }

    fn update_and_get_debuging_data(&mut self, ctx: &Context) -> DebugingDataIn<'_> {
        let color_image = update_and_get_image(
            &mut self.core_game,
        );
        let game_texture_handle = ctx.load_texture(
            "gb_frame",
            color_image,
            TextureOptions::default(),
        );
        debbuger::update_info_struct(self);

        let error_message = if let Some(value) = &self.error_message {
            Some(value)
        } else {
            None
        };
        DebugingDataIn {
            is_step: self.is_step,
            game_texture_handle,
            watched_address: &self.watched_adress,
            registers: &self.registers,
            nb_instruction: self.nb_instruction as u8,
            next_instructions: &self.next_instructions,
            error_message,
            hex_string : &self.hex_string,
        }
    }


    fn switch_state(self, next_state: OutState) -> AppState {
        match next_state {
            OutState::Debuging => {
                AppState::DebugingHub(self)
            }
            OutState::Emulating => {
                AppState::EmulationHub(self.into())
            }
        }
    }
}

