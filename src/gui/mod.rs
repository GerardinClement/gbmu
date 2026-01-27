#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod common;
mod views;

#[derive(Default)]
pub struct MyApp {
    app_state: AppState,
}

use crate::app::GameApp;
use crate::ppu;
use eframe::egui;
use std::sync::Mutex;
use std::fs;
use std::process;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;

use std::sync::{Arc, atomic::AtomicBool};

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app_state = match std::mem::replace(&mut self.app_state, AppState::Default) {
            AppState::StartingHub(device) => device.starting_view(ctx, _frame),
            AppState::SelectionHub(device) => device.selection_view(ctx, _frame),
            AppState::EmulationHub(device) => device.emulation_view(ctx, _frame),
            AppState::DebugingHub(device) => device.debug_view(ctx, _frame),
            AppState::Default => unreachable!(),
        };
        ctx.request_repaint();
    }
}

#[derive(Default)]
pub struct StartingHubDevice {}

pub enum AppState {
    StartingHub(StartingHubDevice),
    SelectionHub(SelectionDevice),
    EmulationHub(EmulationDevice),
    DebugingHub(DebugingDevice),
    Default,
}

fn read_rom(rom_path: String) -> Vec<u8> {
    if !rom_path.is_empty() {
        match fs::read(&rom_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to read the file: {e}");
                process::exit(1);
            }
        }
    } else {
        eprintln!("Failed to read the file: {rom_path} : path is empty");
        process::exit(1);
    }
}

async fn launch_game(
    rom_path: String,
    input_receiver: Receiver<Vec<u8>>,
    image_sender: Sender<bool>,
    command_query_receiver: Receiver<DebugCommandQueries>,
    debug_response_sender: Sender<DebugResponse>,
    global_is_debug: Arc<AtomicBool>,
    image_to_change: Arc<Mutex<Vec<u8>>>,
) {
    let rom_data: Vec<u8> = read_rom(rom_path);
    let mut app = GameApp::new(
        rom_data,
        command_query_receiver,
        debug_response_sender,
        global_is_debug,
        image_to_change,
    );

    loop {
        let buffer = app.update();
        if buffer {
            _ = image_sender.send(buffer).await;
        }
    }
}

pub enum DebugCommandQueries {
    SetStepMode,
    ExecuteInstruction(u8),
    ExecuteNextInstructions(usize),
    GetNextInstructions(u8),
    GetRegisters,
    WatchAddress(u16),
    GetAddresses,
}

pub enum DebugResponse {
    StepModeSet(bool),
    InstructionsExecuted(usize),
    NextInstructions(Vec<u16>),
    AddressesWatched(WatchedAdresses),
    Registers(u8, u8, u8, u8, u8, u8, u8, u16, u16),
}

pub struct WatchedAdresses {
    pub addresses_n_values: Vec<(u16, u16)>,
}

pub struct CoreGameDevice {
    pub handler: JoinHandle<()>,
    pub input_sender: Sender<Vec<u8>>,
    pub image_receiver: Receiver<bool>,
    pub command_query_sender: Sender<DebugCommandQueries>,
    pub debug_response_receiver: Receiver<DebugResponse>,
    pub actual_image: Arc<Mutex<Vec<u8>>>,
    pub resized_image: Vec<u8>,
    pub global_is_debug: Arc<AtomicBool>,
}

use crate::gui::views::emulation_view::scale_image;

impl CoreGameDevice {
    fn new(path: String) -> Self {
        let (input_sender, input_receiver) = channel::<Vec<u8>>(1);
        let (image_sender, image_receiver) = channel::<bool>(1);
        let (command_query_sender, command_query_receiver) = channel::<DebugCommandQueries>(1);
        let (debug_response_sender, debug_response_receiver) = channel::<DebugResponse>(10);
        let global_is_debug = Arc::new(AtomicBool::new(false));
        let actual_image = Arc::new(Mutex::new(vec![0; 160 * 144 * 4]));
        let resized_image = scale_image(&vec![0; 160 * 144 * 4], ppu::WIN_SIZE_X, ppu::WIN_SIZE_Y, 5);
        Self {
            input_sender,
            image_receiver,
            command_query_sender,
            debug_response_receiver,
            handler: tokio::spawn(launch_game(
                path,
                input_receiver,
                image_sender,
                command_query_receiver,
                debug_response_sender,
                global_is_debug.clone(),
                actual_image.clone(),
            )),
            actual_image,
            global_is_debug,
            resized_image,
        }
    }
}

pub struct SelectionDevice {
    path: String,
    files: Vec<String>,
    selected_file: Option<usize>,
}

impl Default for SelectionDevice {
    fn default() -> Self {
        Self {
            path: String::from("./"),
            files: Vec::<String>::default(),
            selected_file: None,
        }
    }
}

pub struct EmulationDevice {
    pub core_game: CoreGameDevice,
}

pub struct DebugingDevice {
    pub core_game: CoreGameDevice,
    /*
    Info stored for the GUI to use them;
    These are the responses from the sending/receiving operation
    */
    pub next_instructions: Vec<u16>,
    pub watched_adress: WatchedAdresses,
    pub registers: (u8, u8, u8, u8, u8, u8, u8, u16, u16),
    pub is_step: bool,
    pub watched_address_value: u16,
    pub nb_instruction: usize,

    pub error_message: Option<String>,
    pub hex_string: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::StartingHub(Default::default())
    }
}
