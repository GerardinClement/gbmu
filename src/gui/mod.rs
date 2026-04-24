#![cfg_attr(test, allow(clippy::all))]
#![allow(unused_variables)]
#![allow(dead_code)]

mod common;
mod views;

use crate::mmu::mbc::{Mbc1, Mbc2, Mbc3, RomOnly};
use crate::{
    ppu,
};
use eframe::egui::{Key, TextureHandle};
use eframe::egui::{load::SizedTexture, vec2, ColorImage, Context, TextureOptions};
use std::collections::HashSet;

use std::sync::atomic::Ordering;

use std::time::Instant;

#[derive(Default)]
pub struct GraphicalApp {
    app_state: AppState,
}

use crate::app::GameApp;
use eframe::egui;
use tokio::time::{self, Duration as TokioDuration};
use std::sync::Mutex;
use std::fs;
use std::process; use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::task::JoinHandle;


use std::sync::{Arc, atomic::AtomicBool};

impl eframe::App for GraphicalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let debut = Instant::now();
        self.app_state = match std::mem::replace(&mut self.app_state, AppState::Default) {
            AppState::StartingHub(device) => device.starting_view(ctx, _frame),
            AppState::SelectionHub(device) => device.selection_view(ctx, _frame),
            AppState::EmulationHub(device) => device.emulation_view(ctx, _frame),
            AppState::DebugingHub(device) => device.debug_view(ctx, _frame),
            AppState::Default => unreachable!(),
        };
        let duration = debut.elapsed();
        //println!("egui : Temps écoulé : {:?} ({} ms)", duration, duration.as_millis());
        ctx.request_repaint();
    }
}

#[derive(Default)]
pub struct StartingHubDevice {}

#[derive(Default, Debug)]
pub struct KeyInput{
    pub a_pushed: bool,
    pub b_pushed: bool,
    pub select_pushed: bool,
    pub start_pushed: bool,
    pub up_pushed: bool,
    pub down_pushed: bool,
    pub left_pushed: bool,
    pub right_pushed: bool,
}


pub struct KeyMaping{
    pub a: Key,
    pub b: Key,
    pub select: Key,
    pub start: Key,
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
}

impl Default for KeyMaping {
    fn default() -> Self {
        KeyMaping {
            a: Key::J,
            b: Key::K,
            select: Key::N,
            start: Key::M,
            up: Key::W,
            down: Key::S,
            left: Key::D,
            right: Key::A,
        }
    }
}


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

pub enum AnyGameApp {
    OnlyRom(GameApp<RomOnly>),
    Mbc1(GameApp<Mbc1>),
    Mbc2(GameApp<Mbc2>),
    Mbc3(GameApp<Mbc3>),
}

impl AnyGameApp {
    pub fn update(&mut self, keys_down: &KeyInput) ->  bool {
        match self {
            AnyGameApp::OnlyRom(g) => g.update(keys_down),
            AnyGameApp::Mbc1(g)=> g.update(keys_down),
            AnyGameApp::Mbc2(g)=> g.update(keys_down),
            AnyGameApp::Mbc3(g)=> g.update(keys_down),

        }
    }
}


async fn launch_game(
    rom_path: String,
    mut input_receiver: Receiver<KeyInput>,
    updated_image_boolean: Arc<AtomicBool>,
    command_query_receiver: Receiver<DebugCommandQueries>,
    debug_response_sender: Sender<DebugResponse>,
    global_is_debug: Arc<AtomicBool>,
    image_to_change: Arc<Mutex<Vec<u8>>>,
) -> Result<(), String> {
    let rom_data: Vec<u8> = read_rom(rom_path);
    let code = rom_data[0x0147];
    let mut app = match code {
            0x00 | 0x08 | 0x09 => Ok(AnyGameApp::OnlyRom(GameApp::new( rom_data, command_query_receiver, debug_response_sender, global_is_debug, image_to_change,)?)),
            0x01 | 0x02 | 0x03 => Ok(AnyGameApp::Mbc1(GameApp::new( rom_data, command_query_receiver, debug_response_sender, global_is_debug, image_to_change,)?)),
            0x05 | 0x06 => Ok(AnyGameApp::Mbc2(GameApp::new( rom_data, command_query_receiver, debug_response_sender, global_is_debug, image_to_change)?)),
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => Ok(AnyGameApp::Mbc3(GameApp::new( rom_data, command_query_receiver, debug_response_sender, global_is_debug, image_to_change)?)),
        /*
            0x0B | 0x0C | 0x0D => Ok(todo!()), // MMM01 pas dans le sujet
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => Ok(todo!()), // Mbc5
            0x20 => Ok(todo!()), // Mbc6
            0x22 => Ok(todo!()),// MBC7+SENSOR+RUMBLE+RAM+BATTERY
        */
            _ => Err("Unmanaged cartridge type")

    }?;

    let input = KeyInput::default();

    loop {
        time::sleep(TokioDuration::from_millis(1)).await;
        // Ceci pourra etre enleve quand on fera
        // du multitask dans le cpu
        // Cela permet de checker si la tache n'a pas ete annule


        if let Ok(input) = input_receiver.try_recv(){
            let input = input;
        }
        let buffer_was_updated = app.update(&input);
        if buffer_was_updated {
            updated_image_boolean.store(true, Ordering::Relaxed);
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
    pub handler: JoinHandle<Result<(), String>>,
    pub input_sender: Sender<KeyInput>,
    pub updated_image_boolean: Arc<AtomicBool>,
    pub command_query_sender: Sender<DebugCommandQueries>,
    pub debug_response_receiver: Receiver<DebugResponse>,
    pub actual_image: Arc<Mutex<Vec<u8>>>,
    pub sized_image: Option<SizedTexture>,
    pub global_is_debug: Arc<AtomicBool>,
    texture_handler: Option<TextureHandle>,
    key_mapping: KeyMaping,
}

impl KeyMaping {
    pub fn generate_key_input(&self, keys_down: HashSet<Key>) -> KeyInput {
        KeyInput {
            a_pushed: keys_down.contains(&self.a),
            b_pushed: keys_down.contains(&self.b),
            select_pushed: keys_down.contains(&self.select),
            start_pushed: keys_down.contains(&self.start),
            up_pushed: keys_down.contains(&self.up),
            down_pushed: keys_down.contains(&self.down),
            left_pushed: keys_down.contains(&self.left),
            right_pushed: keys_down.contains(&self.right),
        }
    }
}

impl Drop for CoreGameDevice {
    fn drop(&mut self) {
        println!("this was droped");
        self.handler.abort();
    }
}

impl CoreGameDevice {

    pub fn update_and_size_image(&mut self, ctx: &Context) {
        if  self.updated_image_boolean.load(Ordering::Relaxed) {
            
            let loaded_image;
            {
                let image = self.actual_image.lock().unwrap();
                loaded_image = ColorImage::from_rgb([ppu::WIN_SIZE_X, ppu::WIN_SIZE_Y], &image);
            }
            if let Some(th) = &mut self.texture_handler {
                th.set(loaded_image, TextureOptions::NEAREST);
            } else {
                self.texture_handler = Some(ctx.load_texture("gb_frame", loaded_image, TextureOptions::NEAREST));
            }
            if let Some(th) = &self.texture_handler {
                let scaled_size = vec2(ppu::WIN_SIZE_X as f32 * 4., ppu::WIN_SIZE_Y as f32 * 4.);
                let sized_texture = SizedTexture::new(th.id(), scaled_size);
                self.sized_image = Some(sized_texture);
                self.updated_image_boolean.store(false, Ordering::Relaxed);
            }
        }
    }

    pub fn capture_input(&self, ctx: &Context) -> KeyInput {
        let keys_down= ctx.input(|i| {
            i.keys_down.clone()
        });
        self.key_mapping.generate_key_input(keys_down)


    }

    fn new(path: String) -> Self {
        let (input_sender, input_receiver) = channel::<KeyInput>(1);
        let updated_image_boolean = Arc::new(AtomicBool::new(false));
        let (command_query_sender, command_query_receiver) = channel::<DebugCommandQueries>(1);
        let (debug_response_sender, debug_response_receiver) = channel::<DebugResponse>(10);
        let global_is_debug = Arc::new(AtomicBool::new(false));
        let actual_image = Arc::new(Mutex::new(vec![0; 160 * 144 * 3]));
        let texture_handler = None;
        Self {
            input_sender,
            command_query_sender,
            debug_response_receiver,
            handler: tokio::spawn(launch_game(
                path,
                input_receiver,
                updated_image_boolean.clone(),
                command_query_receiver,
                debug_response_sender,
                global_is_debug.clone(),
                actual_image.clone(),
            )),
            texture_handler,
            updated_image_boolean,
            actual_image,
            global_is_debug,
            sized_image: None,
            key_mapping: KeyMaping::default(),
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
