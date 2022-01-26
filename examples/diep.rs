use bufro::Color;
use cgmath::VectorSpace;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::time::Instant;
use rand::RngCore;

/***************** STRUCTURES *****************/

pub struct Camera {
    position: cgmath::Vector2<f32>,
}

pub struct Tank {
    position: cgmath::Vector2<f32>,
    velocity: cgmath::Vector2<f32>,
    angle: f32,
    size: f32,
}

impl Tank {
    pub fn update(&mut self) {
        self.position += self.velocity;
        self.velocity *= 0.97; // friction
    }
}

pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Debug)]
pub struct Food {
    position: cgmath::Vector2<f32>,
    velocity: cgmath::Vector2<f32>,
    size: f32,
    sides: u8,
    color: Color,
    angle: f32,
}

impl Food {
    pub fn update(&mut self) {
        self.position += self.velocity;
        self.velocity *= 0.97; // friction

        self.angle += 0.1;
    }
}

/***************** PROCEDURES *****************/

// draw a diep.io style tank
fn draw_tank(painter: &mut bufro::Painter, x: f32, y: f32, size: f32) {
    painter.save();
    painter.translate(x, y);

    // draw a barrel
    let barrel_width = 2.0 * size;
    let barrel_height = 1.0 * size;
    let mut path = bufro::PathBuilder::new();
    path.rectangle(0.0, -barrel_height / 2.0, barrel_width, barrel_height);

    let built = path.build();

    painter.fill_path(&built, Color::from_8(0x99, 0x99, 0x99, 0xff));

    painter.stroke_path(
        &built,
        Color::from_8(0x53, 0x53, 0x53, 0xff),
        bufro::StrokeOptions::default()
            .with_line_join(bufro::LineJoin::Round)
            .with_line_width(4.0),
    );

    // draw an outline
    painter.circle(0.0, 0.0, size + 4.0, Color::from_8(0x53, 0x53, 0x53, 0xff));

    // draw a blue circle
    painter.circle(0.0, 0.0, size, Color::from_8(0x00, 0xB3, 0xE1, 0xff));

    painter.restore();
}

// draw a diep.io style food
fn draw_food(painter: &mut bufro::Painter, x: f32, y: f32, size: f32, color: Color, sides: u8, angle: f32) {
    painter.save();
    painter.translate(x, y);

    // draw a polygon
    let mut path = bufro::PathBuilder::new();
    for i in 0..sides {
        let theta = (i as f32 / sides as f32) * 2. * std::f32::consts::PI;
        let x2 = x + size * (theta + size).cos();
        let y2 = y + size * (theta + size).sin();
        if i == 0 {
            path.move_to(x2, y2);
        } else {
            path.line_to(x2, y2);
        }
    }
    path.close();
    let built = path.build();
    
    painter.rotate(angle);
    painter.fill_path(&built, color);
    painter.stroke_path(
        &built,
        Color::from_8(0x53, 0x53, 0x53, 0xff),
        bufro::StrokeOptions::default()
            .with_line_join(bufro::LineJoin::Round)
            .with_line_width(4.0),
    );

    painter.restore();
}

// draw text with a stroke, size, and font
fn draw_text(
    painter: &mut bufro::Painter,
    font: &bufro::Font,
    x: f32,
    y: f32,
    size: f32,
    text: &str,
) {
    painter.stroke_text(
        &font,
        text,
        x,
        y,
        size,
        Color::from_8(0x2F, 0x2C, 0x30, 0xFF),
        bufro::StrokeOptions::default()
            .with_line_width(5.)
            .with_line_join(bufro::LineJoin::Round),
        None,
    );
    painter.fill_text(
        &font,
        text,
        x,
        y,
        size,
        Color::from_8(0xFF, 0xFF, 0xFF, 0xFF),
        None,
    );
}

/***************** GAME *****************/

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(500, 500))
        .build(&event_loop)
        .unwrap();

    let mut painter = pollster::block_on(bufro::Painter::new_from_window(
        &window,
        (500, 500),
        bufro::Backends::all(),
    ));
    let font = bufro::Font::new(include_bytes!("Overpass-Black.ttf")).unwrap();

    // tesselate a grid
    let mut grid_path = bufro::PathBuilder::new();
    grid_path.move_to(0., 0.);
    for i in 0..500 {
        grid_path.move_to(i as f32 * 20.0, 0.);
        grid_path.line_to(i as f32 * 20.0, 10000.);
    }

    for i in 0..500 {
        grid_path.move_to(0.0, i as f32 * 20.0);
        grid_path.line_to(10000., i as f32 * 20.0);
    }
    grid_path.close();
    let grid_path = grid_path.build();

    let mut camera = Camera {
        position: cgmath::Vector2::new(0.0, 0.0),
    };
    let mut me = Tank {
        position: cgmath::Vector2::new(0.0, 0.0),
        velocity: cgmath::Vector2::new(0.0, 0.0),
        angle: 0.0,
        size: 20.0,
    };

    let mut input = InputState {
        left: false,
        right: false,
        up: false,
        down: false,
    };

    let mut foods = Vec::new();

    // add some random foods
    for _ in 0..1 {
        let x = rand::random::<f32>() * 1000.0;
        let y = rand::random::<f32>() * 1000.0;
        let size = (rand::random::<f32>() * 50.0 + 25.0).round();
        let color = Color::from_8(rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>(), 0xff);
        let sides = (rand::random::<f32>() * 5.0 + 3.0) as u8;
        foods.push(Food {
            position: cgmath::Vector2::new(x, y),
            velocity: cgmath::Vector2::new(0.0, 0.0),
            size,
            color,
            sides,
            angle: 0.0,
        });
    }

    let mut last_frame = Instant::now();

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
                                state,
                                ..
                            },
                        ..
                    } => match keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::Space => {}
                        VirtualKeyCode::W if *state == winit::event::ElementState::Pressed => {
                            input.up = true
                        }
                        VirtualKeyCode::W if *state == winit::event::ElementState::Released => {
                            input.up = false
                        }
                        VirtualKeyCode::A if *state == winit::event::ElementState::Pressed => {
                            input.left = true
                        }
                        VirtualKeyCode::A if *state == winit::event::ElementState::Released => {
                            input.left = false
                        }
                        VirtualKeyCode::S if *state == winit::event::ElementState::Pressed => {
                            input.down = true
                        }
                        VirtualKeyCode::S if *state == winit::event::ElementState::Released => {
                            input.down = false
                        }
                        VirtualKeyCode::D if *state == winit::event::ElementState::Pressed => {
                            input.right = true
                        }
                        VirtualKeyCode::D if *state == winit::event::ElementState::Released => {
                            input.right = false
                        }

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
                let now = Instant::now();
                let delta = (now - last_frame).as_secs_f32() * 1000.0;
                last_frame = now;

                camera.position = camera.position.lerp(me.position, 0.075);
                let size = window.inner_size();
                painter.rectangle(
                    0.0,
                    0.0,
                    size.width as f32,
                    size.height as f32,
                    Color::from_8(0xCD, 0xCD, 0xCD, 0xFF),
                );

                // apply input
                if input.up {
                    me.velocity.y -= 0.1;
                } else if input.down {
                    me.velocity.y += 0.1;
                }

                if input.left {
                    me.velocity.x -= 0.1;
                } else if input.right {
                    me.velocity.x += 0.1;
                }

                me.update();

                // do the camera math
                painter.translate(size.width as f32 / 2.0, size.height as f32 / 2.0);
                painter.translate(-camera.position.x, -camera.position.y);

                // stroke the grid
                painter.stroke_path(
                    &grid_path,
                    Color::from_8(0xC7, 0xC7, 0xC7, 0xFF),
                    bufro::StrokeOptions::default(),
                );

                draw_tank(&mut painter, me.position.x, me.position.y, me.size);

                for food in foods.iter_mut() {
                    food.update();
                    draw_food(&mut painter, food.position.x, food.position.y, food.size, food.color, food.sides, food.angle);
                }
                //draw_food(&mut painter, 0.0, 0.0, 50.0, Color::from_8(0x76, 0x8D, 0xFC, 0xff), 5);

                // undo the camera math
                painter.translate(camera.position.x, camera.position.y);
                painter.translate(-(size.width as f32 / 2.0), -(size.height as f32 / 2.0));

                // 2F2C30
                draw_text(
                    &mut painter,
                    &font,
                    10.,
                    10.,
                    20.,
                    "Diep.io (No bullshit re-tesselation)",
                );

                draw_text(
                    &mut painter,
                    &font,
                    10.,
                    30.,
                    15.,
                    &format!("FPS: {}", (1000.0 / delta).round()),
                );

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
