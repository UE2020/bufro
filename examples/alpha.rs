use bufro::Color;

fn main() {
    unsafe {
        let (mut ctx, window, event_loop) = {
            let event_loop = glutin::event_loop::EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("Bufro Quickstart")
                .with_resizable(true)
                .with_inner_size(glutin::dpi::LogicalSize::new(800., 600.));
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(16)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            let ctx = bufro::Renderer::new(|s| window.get_proc_address(s) as *const _);
            ctx.set_clear_color(Color::from_f(0.2, 0.2, 0.2, 0.2)); // set the bg color

            (ctx, window, event_loop)
        };

        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

            let mut width = 800;
            let mut height = 600;

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
                        ctx.rect(50., 50., 100., 100., 0., Color::from_8(220, 220, 40, 100));
                        ctx.rect(75., 75., 100., 100., 0., Color::from_8(30, 90, 200, 100));

                        ctx.rect(225., 225., 100., 100., 0., Color::from_8(30, 90, 200, 100));
                        ctx.rect(200., 200., 100., 100., 0., Color::from_8(220, 220, 40, 100));

                        ctx.flush();
                        window.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            ctx.resize(physical_size.width as i32, physical_size.height as i32);
                            window.resize(*physical_size);
                            width = physical_size.width;
                            height = physical_size.height;
                        }
                        WindowEvent::CloseRequested => {
                            ctx.clean();
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
