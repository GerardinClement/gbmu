#![allow(unreachable_code)]

use crate::gameboy::GameBoy;
use crate::gui::{DebugCommandQueries, DebugResponse, KeyInput, WatchedAdresses};
use crate::mmu::mbc::Mbc;
use crate::ppu;
use std::sync::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GameApp<T: Mbc> {
    is_debug_mode: Arc<AtomicBool>,
    gameboy: GameBoy<T>,
    debug_receiver: Receiver<DebugCommandQueries>,
    debug_sender: Sender<DebugResponse>,
    is_step_mode: bool,
    nb_next_intruction: u8,
    is_sending_registers: bool,
    watched_adress: WatchedAdresses,
    image_to_change: Arc<Mutex<Vec<u8>>>,
}

impl<T: Mbc> GameApp<T> {
    pub fn new(
        rom: Vec<u8>,
        receiver: Receiver<DebugCommandQueries>,
        sender: Sender<DebugResponse>,
        global_bool: Arc<AtomicBool>,
        image_to_change: Arc<Mutex<Vec<u8>>>,
    ) -> Self {
        let boot_bytes = std::fs::read("boot-roms/dmg.bin").expect("cannot read boot rom");
        assert!(boot_bytes.len() == 0x100, "boot rom must be 256 bytes");

        let mut boot_rom = [0u8; 0x0100];
        boot_rom.copy_from_slice(&boot_bytes);


        let gameboy = GameBoy::<T>::new(rom, boot_rom, image_to_change.clone());
        println!("{}", gameboy.cpu);
        Self {
            gameboy,
            debug_receiver: receiver,
            debug_sender: sender,
            is_step_mode: false,
            is_debug_mode: global_bool,
            is_sending_registers: false,
            nb_next_intruction: 0,
            watched_adress: WatchedAdresses {
                addresses_n_values: Vec::new(),
            },
            image_to_change,
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

    pub fn update(&mut self, keys_down: &KeyInput) -> bool {
        let mut instruction_to_execute = !self.is_step_mode as usize;
        let is_debug = self.is_debug_mode.load(Ordering::Relaxed);
        if is_debug {
            while let Ok(debug) = self.debug_receiver.try_recv() {
                match debug {
                    DebugCommandQueries::ExecuteInstruction(instruction) => {
                        println!("execute instruction received! {instruction}");
                        self.gameboy.cpu.debug_step(instruction);
                        let _ = self
                            .debug_sender
                            .try_send(DebugResponse::InstructionsExecuted(instruction as usize));
                    }
                    DebugCommandQueries::GetNextInstructions(instr_nb) => {
                        println!("get next instruction received! {instr_nb}");
                        self.nb_next_intruction = instr_nb;
                        self.send_next_instructions();
                    }
                    DebugCommandQueries::GetRegisters => {
                        println!("get registers received!");
                        self.is_sending_registers = !self.is_sending_registers;
                        self.send_registers();
                    }
                    DebugCommandQueries::SetStepMode => {
                        println!("set step mode rs received!");
                        self.is_step_mode = !self.is_step_mode;
                        let _ = self
                            .debug_sender
                            .try_send(DebugResponse::StepModeSet(self.is_step_mode));
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
                        instruction_to_execute = nb_instruction;
                        println!("execute next instruction received! {nb_instruction}");
                    }
                    DebugCommandQueries::GetAddresses => {
                        self.send_watched_address();
                    }
                }
            }
        }

        let mut frame_was_edited = false;
        if is_debug {
            for _ in 0..instruction_to_execute {
                frame_was_edited = self.gameboy.run_frame(keys_down);
                self.send_next_instructions();
                self.send_watched_address();
                self.send_registers();
            }
            frame_was_edited
        } else {
            self.gameboy.run_frame(keys_down)
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
