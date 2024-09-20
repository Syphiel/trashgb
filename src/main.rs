mod cpu;
mod mmu;
mod ppu;
mod registers;

use cpu::Cpu;
use pixels::{Pixels, SurfaceTexture};
use std::env;
use std::fs::File;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(160.0, 144.0);
        WindowBuilder::new()
            .with_title("trashgb")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut cpu = Cpu::new();
    let filename = env::args().nth(1).unwrap();
    let game = File::open(filename).unwrap();
    cpu.mmu.load_game(game);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(160, 144, surface_texture).unwrap()
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
