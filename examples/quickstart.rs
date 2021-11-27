use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bufro::Color;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(500, 500))
        .build(&event_loop)
        .unwrap();

    let mut painter = pollster::block_on(bufro::Painter::new_from_window(&window, (500, 500)));
    let font = bufro::Font::new(include_bytes!("FiraMono-Regular.ttf")).unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::Space => {}
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        painter.resize((physical_size.width, physical_size.height));
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        painter.resize((new_inner_size.width, new_inner_size.height));
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let size = window.inner_size();
                painter.rectangle(
                    0.,
                    0.,
                    size.width as f32,
                    size.height as f32,
                    Color::from_f(0.2, 0.2, 0.2, 1.0),
                );
                
                painter.rectangle(0.0, 0.0, 100.0, 100.0, Color::from_f(1.0, 0.0, 0.0, 1.0));
                painter.fill_text(&font, "Hello, World!", 500.0, 500.0, 15.5, Color::from_8(0xFF, 0xFF, 0xFF, 0xFF), None);

                match painter.flush() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(bufro::SurfaceError::Lost) => {
                        painter.clear();
                        painter.regen()
                    }
                    // The system is out of memory, we should probably quit
                    Err(bufro::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => {
                        painter.clear();
                        eprintln!("{:?}", e)
                    }
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
