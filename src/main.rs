mod cpu;
mod ppu;
mod registers;

use cpu::Cpu;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
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
    let game = File::open("./roms/tetris.gb").unwrap();

    for (index, byte) in BufReader::new(game).bytes().enumerate() {
        if index >= 0x100 {
            cpu.memory.push(byte.unwrap());
        }
        else {
            cpu.bootstrap.push(byte.unwrap());
        }
    }
    cpu.memory.resize(0x10000, 0u8);
    cpu.memory[0xFF00] = 0x0F; // TODO: Implement joypad

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(160, 144, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                if cpu.game_loop(pixels.frame_mut()) {
                    pixels.render().unwrap();
                }
            }
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
            Event::RedrawRequested(_) => {
                pixels.render().unwrap();
            }
            _ => (),
        }
    });
}
