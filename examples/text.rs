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
            ctx.set_clear_color(Color::from_f(1., 0., 0., 1.)); // set the bg color

            (ctx, window, event_loop)
        };

        let mut futura =
            bufro::TextRenderer::new(include_bytes!("FuturaPTMedium.otf").to_vec()).unwrap();
        let mut times =
            bufro::TextRenderer::new(include_bytes!("Overpass-Black.ttf").to_vec()).unwrap();
        let demo_text = include_str!("text.txt");

        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

            let mut width = 800;
            let mut height = 600;

            let mut r = 0.;

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
                        //ctx.text(demo_text, Color::from_8(0, 255, 0, 255), &mut futura, 0., 0., 20., width as f32 /2.);
                        //ctx.text(demo_text, Color::from_8(0, 0, 0, 255), &mut times, width as f32 /2. + 10., 0., 20., width as f32 /2. - 50.);
                        ctx.text(
                            "WinFan dumb",
                            Color::from_8(0, 0, 0, 255),
                            &mut times,
                            0.,
                            0.,
                            100.,
                            width as f32 / 2. - 50.,
                        );

                        ctx.translate((width / 2) as f32, (height / 2) as f32);
                        ctx.rotate(r);
                        ctx.polygon(0., 0., 50., 5, Color::from_8(30, 90, 200, 255));

                        // flush
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
