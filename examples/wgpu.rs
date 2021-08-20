use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bufro::Color;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Since main can't be async, we're going to need to block
    let mut painter = pollster::block_on(bufro::Painter::new_from_window(&window));
    painter.set_clear_color(Color::from_f(0.2, 0.2, 0.2, 0.2)); // set the bg color


    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        painter.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        painter.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                painter.rectangle(50., 50., 100., 100., Color::from_8(220, 220, 40, 100));
                painter.rectangle(75., 75., 100., 100., Color::from_8(30, 90, 200, 100));

                painter.rectangle(225., 225., 100., 100., Color::from_8(30, 90, 200, 100));
                painter.rectangle(200., 200., 100., 100., Color::from_8(220, 220, 40, 100));

                painter.translate(window.inner_size().width as f32 / 2., window.inner_size().height as f32 / 2.);

                painter.save();
                painter.translate(0., -75.);
                painter.circle(0., 0., 100., Color::from_8(255, 0, 0, 100));
                painter.restore();

                painter.save();
                painter.translate(75., 0.);
                painter.circle(0., 0., 100., Color::from_8(0, 255, 0, 100));
                painter.restore();

                painter.save();
                painter.translate(-75., 0.);
                painter.circle(0., 0., 100., Color::from_8(0, 0, 255, 100));
                painter.restore();
                match painter.flush() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => painter.regen(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
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
