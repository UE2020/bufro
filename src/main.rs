// Just for testing
mod lib;
use lib as bufro;

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
            let mut r = 0.;
            let mut anim: f32 = 0.;

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
                        r += 0.1;
                        anim += 0.01;
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
                        //ctx.rotate(0.1);
                        //ctx.scale(0.1, 0.1);
                        ctx.translate(0.1, 0.1);
                        ctx.circle(50., 50., 100., bufro::Color::from_8(255, 179, 71, 1));

                        //ctx.rect(300. + anim.sin() * 600., 300., 100., 100., r, bufro::Color::from_8(122, 125, 132, 255));
                        //ctx.triangle(x, y);
                        //ctx.rect(x, y - 70./2., 125. + 9., 70., 1., bufro::Color::from_8(122, 125, 132, 255));
                        //ctx.rect(x, y - 50./2., 125., 50., 1., bufro::Color::from_8(163, 167, 176, 255));
                        //ctx.circle(300. + anim.sin() * 600., y, 60., bufro::Color::from_8(191, 134, 53, 1));
                        //ctx.circle(300. + anim.sin() * 600., y, 50., bufro::Color::from_8(255, 179, 71, 1));


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
