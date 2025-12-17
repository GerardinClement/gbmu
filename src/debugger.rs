#![allow(unused_variables)]
#![allow(dead_code)]



pub mod debbuger {

    use crate::gui::{DebugCommandQueries, DebugResponse, DebugingDevice};

    pub fn update_info_struct(game: &mut DebugingDevice) {
        let count = 0;
        while let Ok(debug) = game.core_game.debug_response_receiver.try_recv() {
            match debug {
                DebugResponse::AddressesWatched(wa) => {
                    println!("DebugResponse::AddressesWatched=> {}", wa.addresses_n_values[0].0);
                    game.watched_adress = wa;
                }
                DebugResponse::StepModeSet(value) => {
                    println!("DebugResponse::StepModeSet=> {value}");
                    game.is_step = value;
                }
                DebugResponse::NextInstructions(list) => {
                    println!("DebugResponse::NextInstruction=> {}", list[0]);
                    game.next_instructions.clear();
                    list.iter().for_each(|f| game.next_instructions.push(*f));
                }
                DebugResponse::InstructionsExecuted(s) => {
                    println!("DebugResponse::InstructionsExecuted=> {s}");
                    todo!();
                }
                DebugResponse::Registers(a, b, c, d, e, h, l, hl, sp) => {
                    println!("DebugResponse::registers=> {a}");
                    game.registers = (a, b, c, d, e, h, l, hl, sp);
                }
            }
        }
    }


    impl DebugingDevice {
        pub fn execute_instruction(&self, instr: u8) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::ExecuteInstruction(instr));
        }

        pub fn get_next_instructions(&self, instr_nb: u8) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::GetNextInstructions(instr_nb));
        }

        pub fn request_registers(&self) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::GetRegisters);
        }

        pub fn request_step_mode(&self) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::SetStepMode);
        }


        pub fn executed_next_step(&self, nb_instru: usize) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::ExecuteNextInstructions(nb_instru));
        }

        pub fn request_watch_address(&self, address: u16) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::WatchAddress(address));
        }

        fn get_watched_addresses(&self) {
            let _ = self
                .core_game
                .command_query_sender
                .try_send(DebugCommandQueries::GetAddresses);
        }
    }
}
