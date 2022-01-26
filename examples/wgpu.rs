use rand::Rng;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bufro::Color;
use cgmath::VectorSpace;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(500, 500))
        .build(&event_loop)
        .unwrap();

    // Since main can't be async, we're going to need to block
    let mut painter = pollster::block_on(bufro::Painter::new_from_window(
        &window,
        (500, 500),
        bufro::Backends::all(),
    ));
    let font = bufro::Font::new(include_bytes!("Roboto-Regular.ttf")).unwrap();
    let mut cursor_position = cgmath::vec2(0.0, 0.0);
    let mut mouse_down = false;

    let mut canvas_translation = cgmath::vec2(0.0, 0.0);
    let mut canvas_translation_lerped = cgmath::vec2(0.0, 0.0);

    let mut canvas_scale = cgmath::vec1(1.);
    let mut canvas_scale_lerped = cgmath::vec1(1.);

    let mut frame = 0;

    //let mut circles = std::collections::HashMap::new();

    let mut rng = rand::thread_rng();

    event_loop.run(move |event, _, control_flow| {
        let text = "Bufro v0.1.7";
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => {
                        let rel_x = cursor_position.x - position.x;
                        let rel_y = cursor_position.y - position.y;
                        if mouse_down {
                            canvas_translation.x -= rel_x / canvas_scale_lerped.x as f64;
                            canvas_translation.y -= rel_y / canvas_scale_lerped.x as f64;
                        }
                        cursor_position = cgmath::vec2(position.x, position.y)
                    }
                    WindowEvent::MouseInput { state, .. } => match state {
                        winit::event::ElementState::Pressed => mouse_down = true,
                        winit::event::ElementState::Released => mouse_down = false,
                    },
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            canvas_scale.x += y / 10.;
                        }
                        _ => {}
                    },
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
                frame += 1;

                let size = window.inner_size();
                painter.rectangle(
                    0.,
                    0.,
                    size.width as f32,
                    size.height as f32,
                    Color::from_f(0.2, 0.2, 0.2, 1.0),
                );
                canvas_translation_lerped = canvas_translation_lerped.lerp(canvas_translation, 0.2);
                canvas_scale_lerped = canvas_scale_lerped.lerp(canvas_scale, 0.2);
                painter.translate(size.width as f32 / 2., size.height as f32 / 2.);
                painter.scale(canvas_scale_lerped.x, canvas_scale_lerped.x);
                painter.translate(
                    -(size.width as f32) / 2. + canvas_translation_lerped.x as f32,
                    -(size.height as f32) / 2. + canvas_translation_lerped.y as f32,
                );
                /*painter.translate(
                    canvas_translation_lerped.x as f32 - (cursor_position.x / 2.) as f32,
                    canvas_translation_lerped.y as f32 - (cursor_position.y / 2.) as f32,
                );
                painter.scale(canvas_scale_lerped.x, canvas_scale_lerped.x);
                painter.translate(
                    -((cursor_position.x / 2.) as f32),
                    -((cursor_position.y / 2.) as f32)
                );*/
                painter.rectangle(50., 50., 100., 100., Color::from_8(220, 220, 40, 100));
                painter.rectangle(75., 75., 100., 100., Color::from_8(30, 90, 200, 100));

                painter.rectangle(225., 225., 100., 100., Color::from_8(30, 90, 200, 100));
                painter.rectangle(200., 200., 100., 100., Color::from_8(220, 220, 40, 100));

                painter.save();
                painter.translate(500., 500.);
                painter.save();
                painter.translate(-75., 0.);
                painter.circle(0., 0., 100., Color::from_8(0, 0, 255, 100));
                painter.restore();

                painter.save();
                painter.translate(75., 0.);
                painter.circle(0., 0., 100., Color::from_8(0, 255, 0, 100));
                painter.restore();

                painter.save();
                painter.translate(0., -75.);
                painter.circle(0., 0., 100., Color::from_8(255, 0, 0, 100));
                painter.restore();
                painter.restore();

                painter.save();
                painter.translate(700., 100.);
                let mut path = bufro::PathBuilder::new();
                path.move_to(0., 0.);
                path.quad_to(100., 100., 200., 0.);
                path.quad_to(100., 100., 200., 200.);
                path.quad_to(100., 100., 0., 200.);
                path.quad_to(100., 100., 0., 0.);
                path.close();
                let path = path.build();
                painter.fill_path(&path, Color::from_8(200, 200, 200, 255));
                painter.stroke_path(
                    &path,
                    Color::from_8(255, 255, 255, 255),
                    bufro::StrokeOptions::default()
                        .with_line_width(10.)
                        .with_line_join(bufro::LineJoin::Round),
                );
                painter.translate(300., 0.);
                let mut path = bufro::PathBuilder::new();
                path.move_to(0., 0.);
                path.curve_to(100., 100., 100., -100., 200., 0.);
                path.curve_to(100., 100., 300., 100., 200., 200.);
                path.curve_to(100., 100., 100., 300., 0., 200.);
                path.curve_to(100., 100., -100., 100., 0., 0.);
                path.close();
                let path = path.build();
                painter.fill_path(&path, Color::from_8(200, 200, 200, 255));
                painter.stroke_path(
                    &path,
                    Color::from_8(255, 255, 255, 255),
                    bufro::StrokeOptions::default()
                        .with_line_width(10.)
                        .with_line_join(bufro::LineJoin::Round),
                );

                painter.restore();

                /*if circles.len() < 10000 {
                    circles.insert(
                        frame,
                        (
                            rng.gen_range(0.0..10000.0),
                            rng.gen_range(0.0..10000.0),
                            rng.gen_range(10.0..50.0),
                        ),
                    );
                }

                let time_start = std::time::Instant::now();
                for (_, circle) in circles.iter_mut() {
                    painter.save();
                    painter.translate(circle.0, circle.1);
                    //painter.circle(0.0, 0.0, circle.2 + 5.0, Color::from_8(174, 63, 0, 255));
                    painter.circle(0.0, 0.0, circle.2, Color::from_8(214, 73, 5, 255));
                    painter.restore();
                }
                println!("Elapsed: {:?}", time_start.elapsed());*/

                painter.circle(
                    0.,
                    0.,
                    (frame % 60 + 1) as f32,
                    Color::from_8(255, 255, 255, 255),
                );

                painter.reset();
                painter.fill_text(
                    &font,
                    &painter.get_buffer_info(),
                    0.,
                    0.,
                    15.5,
                    Color::from_8(0xFF, 0xFF, 0xFF, 0xFF),
                    Some(150),
                );

                match painter.flush() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        painter.clear();
                        painter.regen()
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
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
