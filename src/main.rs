mod cpu;
mod mmu;
mod ppu;
mod registers;

use cpu::Cpu;
use pixels::{Pixels, SurfaceTexture};
use std::fs::File;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(160.0, 144.0);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut cpu = Cpu::new();
    // let game = File::open("./roms/tests/02-interrupts.gb").unwrap();
    let game = File::open("./roms/tetris.gb").unwrap();
    cpu.mmu.load_game(game);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(160, 144, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
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
            Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));
                if cpu.game_loop(pixels.frame_mut()) {
                    pixels.render().unwrap();
                }
            }
            Event::RedrawRequested(_) => { } _ => {}
        }
    });
}
