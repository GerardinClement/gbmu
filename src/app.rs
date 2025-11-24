#![allow(unreachable_code)]

use crate::ppu;
use crate::{DebugCommandQueries, DebugResponse, WatchedAdresses, gameboy::GameBoy};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GameApp {
    gameboy: GameBoy,
    framebuffer: Vec<u8>,
    debug_receiver: Receiver<DebugCommandQueries>,
    debug_sender: Sender<DebugResponse>,
    is_step_mode: bool,
    is_debug_mode: bool,
    nb_next_intruction: u8,
    is_sending_registers: bool,
    watched_adress: WatchedAdresses,
}

impl GameApp {
    pub fn new(
        rom: Vec<u8>,
        receiver: Receiver<DebugCommandQueries>,
        sender: Sender<DebugResponse>,
    ) -> Self {
        let gameboy = GameBoy::new(rom);
        println!("{}", gameboy.cpu);
        Self {
            gameboy,
            framebuffer: vec![0; 160 * 144 * 4],
            debug_receiver: receiver,
            debug_sender: sender,
            is_step_mode: false,
            is_debug_mode: false,
            is_sending_registers: false,
            nb_next_intruction: 0,
            watched_adress: WatchedAdresses {
                addresses_n_values: Vec::new(),
            },
        }
    }

    fn send_watched_address(&mut self) {
        if !self.watched_adress.addresses_n_values.is_empty() {
            let mut values = WatchedAdresses {
                addresses_n_values: Vec::new(),
            };
            for (addr, _) in self.watched_adress.addresses_n_values.iter() {
                if let Ok(bus) = self.gameboy.bus.read() {
                    let value = bus.read_byte(*addr);
                    values.addresses_n_values.push((*addr, value as u16));
                }
            }
            let _ = self
                .debug_sender
                .try_send(DebugResponse::AddressesWatched(values));
        }
        //65408
    }

    fn send_registers(&mut self) {
        let _ = self.debug_sender.try_send(DebugResponse::Registers(
            self.gameboy.cpu.registers.get_a(),
            self.gameboy.cpu.registers.get_b(),
            self.gameboy.cpu.registers.get_c(),
            self.gameboy.cpu.registers.get_d(),
            self.gameboy.cpu.registers.get_e(),
            self.gameboy.cpu.registers.get_h(),
            self.gameboy.cpu.registers.get_flags_u8(),
            self.gameboy.cpu.registers.get_sp(),
            self.gameboy.cpu.pc,
        ));
    }

    fn send_next_instructions(&mut self) {
        let mut v = vec![self.gameboy.cpu.pc];
        for current_instruction in 1..self.nb_next_intruction {
            v.push(
                self.gameboy
                    .bus
                    .read()
                    .unwrap()
                    .read_byte(self.gameboy.cpu.pc.wrapping_add(current_instruction as u16))
                    as u16,
            );
        }
        let _ = self
            .debug_sender
            .try_send(DebugResponse::NextInstructions(v));
    }

    pub fn update(&mut self) -> Option<Vec<u8>> {
        if let Ok(debug) = self.debug_receiver.try_recv() {
            match debug {
                DebugCommandQueries::ExecuteInstruction(instruction) => {
                    self.gameboy.cpu.debug_step(instruction);
                    let _ = self
                        .debug_sender
                        .try_send(DebugResponse::InstructionsExecuted(instruction as usize));
                }
                DebugCommandQueries::GetNextInstructions(instr_nb) => {
                    self.nb_next_intruction = instr_nb;
                    self.send_next_instructions();
                }
                DebugCommandQueries::GetRegisters => {
                    self.is_sending_registers = !self.is_sending_registers;
                    self.send_registers();
                }
                DebugCommandQueries::SetStepMode => {
                    self.is_step_mode = !self.is_step_mode;
                    let _ = self
                        .debug_sender
                        .try_send(DebugResponse::StepModeSet(self.is_step_mode));
                }
                DebugCommandQueries::SetDebugMode => {
                    self.is_debug_mode = !self.is_debug_mode;
                    let _ = self
                        .debug_sender
                        .try_send(DebugResponse::DebugModeSet(self.is_debug_mode));
                }
                DebugCommandQueries::WatchAddress(addr) => {
                    if !self
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .any(|(a, _)| *a == addr)
                    {
                        self.watched_adress.addresses_n_values.push((addr, 0));
                    } else if let Some(index) = self
                        .watched_adress
                        .addresses_n_values
                        .iter()
                        .position(|(address, _)| *address == addr)
                    {
                        self.watched_adress.addresses_n_values.remove(index);
                    }
                    self.send_watched_address();
                }
                DebugCommandQueries::ExecuteNextInstructions(nb_instruction) => {
                    let mut last_frame = None;
                    for _ in 0..nb_instruction {
                        let rgb_frame = self.gameboy.run_frame();
                        last_frame = Some(Self::rgb_to_rgba(&rgb_frame));
                    }
                    return last_frame;
                }
                DebugCommandQueries::GetAddresses => {
                    self.send_watched_address();
                }
            }
        }

        if !self.is_step_mode {
            let rgb_frame = self.gameboy.run_frame();
            Some(Self::rgb_to_rgba(&rgb_frame))
        } else {
            if self.nb_next_intruction != 0 {
                self.send_next_instructions();
            };
            self.send_watched_address();
            self.send_registers();
            None
        }
    }

    fn rgb_to_rgba(rgb_frame: &[u8]) -> Vec<u8> {
        let mut rgba_frame = Vec::with_capacity(ppu::WIN_SIZE_X * ppu::WIN_SIZE_Y * 4);
        for chunk in rgb_frame.chunks(3) {
            rgba_frame.extend_from_slice(chunk);
            rgba_frame.push(255);
        }
        rgba_frame
    }
}
