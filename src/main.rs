#![deny(clippy::all)]

mod cpu;
mod mapper;
mod mmu;
mod ppu;
mod registers;

use cpu::Cpu;

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use instant::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom>", args[0]);
        std::process::exit(1);
    }
    let rom = std::fs::read(&args[1]).unwrap();
    pollster::block_on(run(&rom));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(rom: &[u8]) {
    let rom: &'static [u8] = Box::leak(rom.to_vec().into_boxed_slice());
    wasm_bindgen_futures::spawn_local(run(rom));
}

async fn run(rom: &[u8]) {
    let rom = std::io::Cursor::new(rom);
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(640.0, 576.0);
        WindowBuilder::new()
            .with_title("trashgb")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    #[cfg(target_arch = "wasm32")]
    {
        let canvas = window.canvas();
        web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(canvas))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
    }
    let mut cpu = Cpu::new();
    cpu.mmu.load_game(rom);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new_async(160, 144, surface_texture).await.unwrap()
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {}
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            let _ = pixels.resize_surface(size.width, size.height);
        }
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));
            pixels.render().unwrap();
        }
        Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));
            if cpu.game_loop(pixels.frame_mut()) {
                pixels.render().unwrap();
            }
        }
        // Keyboard Input
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            if let Some(key) = input.virtual_keycode {
                match input.state {
                    winit::event::ElementState::Pressed => match key {
                        VirtualKeyCode::Up => cpu.mmu.joypad_up(true),
                        VirtualKeyCode::Down => cpu.mmu.joypad_down(true),
                        VirtualKeyCode::Left => cpu.mmu.joypad_left(true),
                        VirtualKeyCode::Right => cpu.mmu.joypad_right(true),
                        VirtualKeyCode::Z => cpu.mmu.joypad_a(true),
                        VirtualKeyCode::X => cpu.mmu.joypad_b(true),
                        VirtualKeyCode::Return => cpu.mmu.joypad_start(true),
                        VirtualKeyCode::Back => cpu.mmu.joypad_select(true),
                        _ => {}
                    },
                    winit::event::ElementState::Released => match key {
                        VirtualKeyCode::Up => cpu.mmu.joypad_up(false),
                        VirtualKeyCode::Down => cpu.mmu.joypad_down(false),
                        VirtualKeyCode::Left => cpu.mmu.joypad_left(false),
                        VirtualKeyCode::Right => cpu.mmu.joypad_right(false),
                        VirtualKeyCode::Z => cpu.mmu.joypad_a(false),
                        VirtualKeyCode::X => cpu.mmu.joypad_b(false),
                        VirtualKeyCode::D => println!("{:08b}", cpu.mmu.read_byte(0xFF41)),
                        VirtualKeyCode::Return => cpu.mmu.joypad_start(false),
                        VirtualKeyCode::Back => cpu.mmu.joypad_select(false),
                        _ => {}
                    },
                }
            }
        }
        Event::RedrawRequested(_) => {}
        _ => {}
    });
}
