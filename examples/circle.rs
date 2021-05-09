// Just for testing

#[derive(Default)]
struct Keys {
    W: bool,
    A: bool,
    S: bool,
    D: bool
}

fn main() {
    unsafe {
        let (mut ctx, window, event_loop) = {
            let event_loop = glutin::event_loop::EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_resizable(false)
                .with_inner_size(glutin::dpi::LogicalSize::new(700.0, 700.0));
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(16)
                //.with_srgb(true)
                //.with_pixel_format(24, 8)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            let ctx = bufro::Renderer::new(|s| window.get_proc_address(s) as *const _);
            (ctx, window, event_loop)
        };

        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;
            let mut keys: Keys = Default::default();
            let mut x = 0.;
            let mut y = 0.;

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::LoopDestroyed => {
                        return;
                    }
                    Event::MainEventsCleared => {
                        window.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {
                        if keys.A {
                            x -= 5.;
                        }
                        else if keys.D {
                            x += 5.;
                        }

                        if keys.W {
                            y -= 5.;
                        } else if keys.S {
                            y += 5.;
                        }

                        //ctx.triangle(x, y);
                        ctx.circle(x, y, 60., bufro::Color::from_8(191, 134, 53, 1));
                        ctx.circle(x, y, 50., bufro::Color::from_8(255, 179, 71, 1));

                        ctx.flush();
                        window.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            ctx.resize(physical_size.width as i32, physical_size.height as i32);
                            window.resize(*physical_size);
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            //x = position.x as f32;
                            //y = position.y as f32;
                            //println!("x: {} y: {}", x, y);
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            if !input.virtual_keycode.is_none() {
                                if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::A && input.state == glutin::event::ElementState::Pressed {
                                    keys.A = true;
                                } else if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::A && input.state == glutin::event::ElementState::Released {
                                    keys.A = false;
                                }
    
                                if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::D && input.state == glutin::event::ElementState::Pressed {
                                    keys.D = true;
                                } else if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::D && input.state == glutin::event::ElementState::Released {
                                    keys.D = false;
                                }
    
                                if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::W && input.state == glutin::event::ElementState::Pressed {
                                    keys.W = true;
                                } else if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::W && input.state == glutin::event::ElementState::Released {
                                    keys.W = false;
                                }
    
                                if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::S && input.state == glutin::event::ElementState::Pressed {
                                    keys.S = true;
                                } else if input.virtual_keycode.unwrap() == glutin::event::VirtualKeyCode::S && input.state == glutin::event::ElementState::Released {
                                    keys.S = false;
                                }
                            }
                        }
                        WindowEvent::CloseRequested => {
                            //ctx.destroy();
                            *control_flow = ControlFlow::Exit
                        }
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    }
}
