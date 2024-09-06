mod cpu;
mod ppu;
mod registers;

use cpu::Cpu;
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
    cpu.memory.resize(0xFFFF, 0u8);

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
                // cpu.render_frame(pixels.frame_mut());
                pixels.render().unwrap();
            }
            _ => (),
        }
    });
}

// fn render(frame: &mut [u8]) {
//     for pixel in frame.chunks_exact_mut(4) {
//         pixel.copy_from_slice(&[0xcc, 0, 0xcc, 255]);
//     }
// }
