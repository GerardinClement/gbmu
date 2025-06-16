use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;

use crate::gameboy::{GameBoy};

#[derive(Default)]
pub struct App<'win> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'win>>,
	gameboy: GameBoy,
    framebuffer: Vec<u8>,
}

impl App<'_> {
	pub fn new(rom: Vec<u8>) -> Self {
		App {
			window: None,
			pixels: None,
			gameboy: GameBoy::new(rom),
			framebuffer: vec![0; 160 * 144 * 4],
		}
	}

	pub fn update(&mut self) {
		if let Some(window) = self.window.as_ref() {
			self.gameboy.cpu.step();
			let rgb_frame = self.gameboy.ppu.render_frame();
			self.framebuffer = App::rgb_to_rgba(&rgb_frame);
			Window::request_redraw(window);
		} else {
			eprintln!("Error: Window is not initialized.");
		}
	}
	
	fn rgb_to_rgba(rgb_frame: &[u8]) -> Vec<u8> {
		let mut rgba_frame = Vec::with_capacity(160 * 144 * 4);
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
		event_loop.set_control_flow(ControlFlow::WaitUntil(
			std::time::Instant::now() + std::time::Duration::from_millis(16),
		));
	}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let size = window.inner_size();
    
        self.window = Some(window.clone());
    
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(160, 144, surface_texture).unwrap();
    
        self.pixels = Some(pixels);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
				println!("{:?}", self.gameboy.ppu.display_vram());
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let pixels = self.pixels.as_mut().unwrap();
                let frame = pixels.frame_mut();

				frame.copy_from_slice(&self.framebuffer);
    
                pixels.render();
            }
            _ => (),
        }
    }
    
}
