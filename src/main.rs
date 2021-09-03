mod chip8;
mod monitor;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::chip8::Chip8;
use crate::monitor::Monitor;
use crate::monitor::{WIDTH, HEIGHT};

const SCALING: u32 = 20;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as u32 * SCALING, HEIGHT as u32 * SCALING);
        WindowBuilder::new()
            .with_title("Chip8 emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut monitor = Monitor::new();

    let mut pixels = {
        let windows_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(windows_size.width, windows_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            // redraw monitor
            draw_monitor(&mut monitor, pixels.get_frame());
            // monitor draw screen.

            if pixels
                .render()
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;   
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_buffer(size.width, size.height);
            }

            // Resize the window 
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            //Update internal state and request a redraw

        }

    })
}

fn draw_monitor(monitor: &mut Monitor, frame: &mut [u8]) {
    monitor.set_pixel(0, 0);
    monitor.set_pixel(1, 1);
    monitor.set_pixel(2, 2);
    monitor.set_pixel(3, 3);
    monitor.set_pixel(6, 5);
    monitor.set_pixel(6, 4);
    monitor.set_pixel(6, 4);

    let black = [0x00, 0x00, 0x00, 0xff];
    let white = [0xFF, 0xFF, 0xFF, 0xff];

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let binary_pixel = monitor.pixels[i];
        if binary_pixel {
            pixel.copy_from_slice(&white);
        } else {
            pixel.copy_from_slice(&black);
        }
    } 
}