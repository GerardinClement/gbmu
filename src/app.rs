#![allow(unreachable_code)]

use crate::gameboy::GameBoy;
use crate::gui::{DebugCommandQueries, DebugResponse, WatchedAdresses};
use crate::ppu;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GameApp {
    is_debug_mode: Arc<AtomicBool>,
    gameboy: GameBoy,
    debug_receiver: Receiver<DebugCommandQueries>,
    debug_sender: Sender<DebugResponse>,
    is_step_mode: bool,
    nb_next_intruction: u8,
    is_sending_registers: bool,
    watched_adress: WatchedAdresses,
}

impl GameApp {
    pub fn new(
        rom: Vec<u8>,
        receiver: Receiver<DebugCommandQueries>,
        sender: Sender<DebugResponse>,
        global_bool: Arc<AtomicBool>,
    ) -> Self {
        let gameboy = GameBoy::new(rom);
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
        println!("update");
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

        let mut last_frame = None;
        if is_debug {
            for _ in 0..instruction_to_execute {
                let rgb_frame = self.gameboy.run_frame();
                self.send_next_instructions();
                self.send_watched_address();
                self.send_registers();
                last_frame = Some(Self::rgb_to_rgba(&rgb_frame));
            }
            last_frame
        } else {
            let rgb_frame = self.gameboy.run_frame();
            Some(Self::rgb_to_rgba(&rgb_frame))
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

impl ApplicationHandler for App<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        match cause {
            winit::event::StartCause::Init => {
                self.resumed(event_loop);
            }
            winit::event::StartCause::ResumeTimeReached { .. } => {
                self.update();
            }
            _ => (),
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(std::time::Instant::now()));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let size = window.inner_size();

        self.window = Some(window.clone());

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(160, 144, surface_texture).unwrap();

        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // println!("{:?}", self.gameboy.ppu.display_vram());
                self.gameboy.ppu.display_tile_map_area(0x9800);
                self.gameboy.ppu.display_tile_map_area(0x9C00);
                self.gameboy.ppu.display_tiles_data();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let pixels = self.pixels.as_mut().unwrap();
                let frame = pixels.frame_mut();

                frame.copy_from_slice(&self.framebuffer);

                let _ = pixels.render();
            }
            _ => (),
        }
    }
}
