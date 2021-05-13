use cgmath::SquareMatrix;
use cgmath::Zero;

use glow::*;

use std::convert::AsRef;
use std::f32::consts::PI;

pub mod ffi;

/// Represents a color with values from 0-1
#[derive(Clone)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }
}

/// A renderer command, all commands are rendered upon flush
enum Command {
    Triangle {
        x: f32,
        y: f32,
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
        color: Color,
        transform: cgmath::Matrix4<f32>,
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        angle: f32,
        color: Color,
        transform: cgmath::Matrix4<f32>,
    },
    Polygon {
        x: f32,
        y: f32,
        r: f32,
        sides: u8,
        color: Color,
        transform: cgmath::Matrix4<f32>,
    }
}

#[derive(Clone)]
pub struct Transform {
    rotation: f32,
    translation: cgmath::Vector2<f32>,
    scale: cgmath::Vector2<f32>,
}

/// A basic window-less renderer (though you can always just load the function pointers of a window)
pub struct Renderer {
    gl: Context,                 // the main context
    program: u32,                // the main shader
    command_stack: Vec<Command>, // clear command_stack on flush,
    projection: cgmath::Matrix4<f32>,

    // caching
    vertex_array: u32,
    vertex_buffer: u32,
    index_buffer: u32,

    // matrix
    transform: cgmath::Matrix4<f32>,
    old_transform: cgmath::Matrix4<f32>,
}

impl Renderer {
    pub fn new<F: FnMut(&str) -> *const std::os::raw::c_void>(function_loader: F) -> Self {
        unsafe {
            let gl = glow::Context::from_loader_function(function_loader); // load function pointers

            let program = gl.create_program().expect("Cannot create program"); // compile and link shader program

            let (vertex_shader_source, fragment_shader_source) = (
                r#"layout(location = 0) in vec2 vertexPosition_modelspace;
                //out vec2 vert;
                uniform mat4 transform;

                void main() {
                    gl_Position = transform * vec4(vertexPosition_modelspace, 0.0, 1.0);
                    //gl_Position.xy += pos;
                    gl_Position.w = 1.0;
                }"#,
                r#"
                out vec4 color;
                uniform vec4 col;
                void main(){
                    color = col;
                }"#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, &format!("{}\n{}", "#version 410", shader_source));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    std::panic::panic_any(gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                std::panic::panic_any(gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.use_program(Some(program));
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            let mut m_viewport: [i32; 7] = [0; 7];

            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut m_viewport);

            let vertex_array = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vertex_array));

            let vertex_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

            let index_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);

            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);  

            //gl.polygon_mode(glow::FRONT, glow::LINE);
            //gl.polygon_mode(glow::BACK, glow::LINE);

            Self {
                gl,
                program,
                command_stack: Vec::new(), // Renderers should start with no pending commands
                projection: cgmath::ortho(
                    0.,
                    m_viewport[2] as f32,
                    m_viewport[3] as f32,
                    0.,
                    0.,
                    1.,
                ),

                vertex_array,
                vertex_buffer,
                index_buffer,

                transform: cgmath::Matrix4::identity(),
                old_transform: cgmath::Matrix4::identity(),
            }
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            for command in self.command_stack.drain(0..) {
                match command {
                    Command::Triangle { x, y } => {
                        let vertex_array = self.gl.create_vertex_array().unwrap();
                        self.gl.bind_vertex_array(Some(vertex_array));

                        let vertex_buffer_data = [-50., -50., 0., 50., -50., 0., 0., 50., 0.];

                        let vertex_buffer = self.gl.create_buffer().unwrap();
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
                        self.gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            &std::mem::transmute::<[f32; 9], [u8; 36]>(vertex_buffer_data),
                            glow::DYNAMIC_DRAW,
                        );

                        self.gl.enable_vertex_attrib_array(0);
                        self.gl
                            .vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::zero();
                        mat.w.x = x;
                        mat.w.y = y;
                        let final_mat = mat + self.projection;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);
                        self.gl.draw_arrays(glow::TRIANGLES, 0, 3);

                        self.gl.delete_vertex_array(vertex_array);
                        self.gl.delete_buffer(vertex_buffer);
                    }
                    Command::Circle {
                        x,
                        y,
                        r,
                        color,
                        transform,
                    } => {
                        let max = PI * 2.;
                        let mut vertices = Vec::with_capacity(max as usize + 1);
                        let mut i = 0.;
                        let points_calculation = r * 0.64;
                        let points = if points_calculation > 10. { points_calculation } else { 10. };
                        while i < max {
                            vertices.push(i.cos() * r);
                            vertices.push(i.sin() * r);
                            i += max / points;
                        }
                        vertices.push(i.cos() * r);
                        vertices.push(i.sin() * r);

                        let mut vertex_buffer_data = Vec::<u8>::with_capacity(vertices.len() * 4);
                        for float in vertices.iter() {
                            vertex_buffer_data.extend_from_slice(&float.to_le_bytes());
                        }

                        self.gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            vertex_buffer_data.as_ref(),
                            glow::DYNAMIC_DRAW,
                        );

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        mat.w.x = x;
                        mat.w.y = y;

                        let final_mat = self.projection * transform * mat;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

                        self.gl
                            .draw_arrays(glow::TRIANGLE_FAN, 0, vertices.len() as i32);
                    }
                    Command::Polygon {
                        x,
                        y,
                        r,
                        color,
                        sides,
                        transform,
                    } => {
                        let max = PI * 2.;
                        let mut vertices = Vec::with_capacity(max as usize + 1);
                        let mut i = 0.;
                        let points = sides as f32;
                        while i < max {
                            vertices.push(i.cos() * r);
                            vertices.push(i.sin() * r);
                            i += max / points;
                        }
                        vertices.push(i.cos() * r);
                        vertices.push(i.sin() * r);

                        let mut vertex_buffer_data = Vec::<u8>::with_capacity(vertices.len() * 4);
                        for float in vertices.iter() {
                            vertex_buffer_data.extend_from_slice(&float.to_le_bytes());
                        }

                        self.gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            vertex_buffer_data.as_ref(),
                            glow::DYNAMIC_DRAW,
                        );

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        mat.w.x = x;
                        mat.w.y = y;

                        let final_mat = self.projection * transform * mat;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

                        self.gl
                            .draw_arrays(glow::TRIANGLE_FAN, 0, vertices.len() as i32);
                    }
                    Command::Rectangle {
                        x,
                        y,
                        width,
                        height,
                        color,
                        angle,
                        transform,
                    } => {
                        let vertex_buffer_data = [
                            0., 0., width, height, 0., height, //
                            width, 0., width, height, 0., 0.,
                        ];

                        self.gl.buffer_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            &std::mem::transmute::<[f32; 12], [u8; 48]>(vertex_buffer_data),
                            glow::DYNAMIC_DRAW,
                        );

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        mat.w.x = x;
                        mat.w.y = y;
                        mat.x.x = angle.cos();
                        mat.x.y = -angle.sin();
                        mat.y.x = angle.sin();
                        mat.y.y = angle.cos();

                        let final_mat = self.projection * transform * mat;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

                        self.gl
                            .draw_arrays(glow::TRIANGLES, 0, vertex_buffer_data.len() as i32);
                    }
                }
            }

            self.transform = cgmath::Matrix4::identity();
        }
    }

    pub fn triangle(&mut self, x: f32, y: f32) {
        self.command_stack.push(Command::Triangle { x, y })
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, angle: f32, color: Color) {
        self.command_stack.push(Command::Rectangle {
            x,
            y,
            width,
            height,
            color,
            angle,
            transform: self.transform.clone(),
        })
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32, color: Color) {
        self.command_stack.push(Command::Circle {
            x,
            y,
            r,
            color,
            transform: self.transform.clone(),
        })
    }

    pub fn polygon(&mut self, x: f32, y: f32, r: f32, sides: u8, color: Color) {
        self.command_stack.push(Command::Polygon {
            x,
            y,
            r,
            color,
            sides,
            transform: self.transform.clone(),
        })
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            let _size = 200.;
            let _aspect = width as f32 / height as f32;
            self.projection = cgmath::ortho(0., width as f32, height as f32, 0., 0., 1.);
            self.gl.viewport(0, 0, width, height); // resize viewport
        }
    }

    pub fn clean(&self) {
        unsafe {
            self.gl.delete_program(self.program); // clean up program
            self.gl.delete_vertex_array(self.vertex_array); // clean up buffers
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.index_buffer);
        }
    }

    pub fn set_clear_color(&self, color: Color) {
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
        }
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.transform = self.transform * cgmath::Matrix4::from_nonuniform_scale(x, y, 1.);
    }

    pub fn rotate(&mut self, x: f32) {
        self.transform = self.transform * cgmath::Matrix4::from_angle_z(cgmath::Rad(x));
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform = self.transform * cgmath::Matrix4::from_translation(cgmath::vec3(x, y, 0.));
    }

    pub fn save(&mut self) {
        self.old_transform = self.transform;
    }

    pub fn restore(&mut self) {
        self.transform = self.old_transform;
    }

    pub fn reset(&mut self) {
        self.transform = cgmath::Matrix4::identity();
    }
}
