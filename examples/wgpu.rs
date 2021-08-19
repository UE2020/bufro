use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Since main can't be async, we're going to need to block
    let mut painter = pollster::block_on(bufro::Painter::new_from_window(&window));

    let mut rotation = 0.;

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
                        //painter.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        //painter.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                painter.resize(window.inner_size());
                painter.resize(window.inner_size());
                painter.save();
                painter.translate(rotation, 200.);
                painter.rotate(rotation);
                rotation += 0.1;
                painter.rectangle(-50., -50., 100., 100., bufro::Color::from_8(255, 155, 55, 255));
                painter.restore();
                painter.save();
                painter.translate(rotation, 200.);
                painter.rotate(rotation);
                painter.rectangle(-25., -25., 50., 50., bufro::Color::from_8(55, 155, 255, 255));
                painter.restore();
                painter.circle(200., 200., 100., bufro::Color::from_8(55, 155, 255, 255));
                painter.reset();
                match painter.flush() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => painter.resize(painter.size),
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
