use glow::*;
use cgmath::Zero;
use std::convert::AsRef;
use std::f32::consts::PI;
use cgmath::SquareMatrix;

/// Represents a color with values from 0-1
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {
    pub fn from_f (r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r, g, b, a
        }
    }

    pub fn from_8 (r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }
}

/// A renderer command, all commands are rendered upon flush
pub enum Command {
    Triangle {
        x: f32,
        y: f32
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
        color: Color
    }
}

/// A basic window-less renderer (though you can always just load the function pointers of a window)
pub struct Renderer {
    gl: Context, // the main context
    program: u32, // the main shader
    command_stack: Vec<Command>, // clear command_stack on flush,
    projection: cgmath::Matrix4<f32>
}

impl Renderer {
    pub fn new<F: FnMut(&str) -> *const std::os::raw::c_void> (function_loader: F) -> Self {
        unsafe {
            let gl = glow::Context::from_loader_function(function_loader); // load function pointers

            let program = gl.create_program().expect("Cannot create program"); // compile and link shader program

            let (vertex_shader_source, fragment_shader_source) = (
                r#"layout(location = 0) in vec3 vertexPosition_modelspace;
                //out vec2 vert;
                uniform mat4 transform;

                void main() {
                    gl_Position = transform * vec4(vertexPosition_modelspace, 1.0);
                    //gl_Position.xy += pos;
                    gl_Position.w = 1.0;
                }"#,
                r#"
                out vec3 color;
                uniform vec3 col;
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
                    panic!(gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!(gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.use_program(Some(program));
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            let mut m_viewport: [i32; 7] = [0; 7];

            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut m_viewport);
            println!("parameter {:?}", m_viewport);
            let size = 200.;
            let aspect = m_viewport[2] as f32 / m_viewport[3] as f32;
            

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
                    1.
                )
            }
        }
    }

    pub fn flush (&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            for command in self.command_stack.drain(0..) {
                match command {
                    Command::Triangle { x, y } => {
                        let vertex_array = self.gl.create_vertex_array().unwrap();
                        self.gl.bind_vertex_array(Some(vertex_array));

                        let vertex_buffer_data = [
                            -50., -50., 0.,
                            50., -50., 0.,
                            0., 50., 0.
                        ];

                        let vertex_buffer = self.gl.create_buffer().unwrap();
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
                        self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &std::mem::transmute::<[f32; 9], [u8; 36]>(vertex_buffer_data), glow::STATIC_DRAW);

                        self.gl.enable_vertex_attrib_array(0);
                        self.gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                        let loc = &self.gl.get_uniform_location(self.program, "transform").unwrap();

                        let mut mat = cgmath::Matrix4::zero();
                        mat.w.x = x;
                        mat.w.y = y;
                        let final_mat = mat + self.projection;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);
                        self.gl.draw_arrays(glow::TRIANGLES, 0, 3);

                        self.gl.delete_vertex_array(vertex_array);
                        self.gl.delete_buffer(vertex_buffer);
                    },
                    Command::Circle { x, y, r, color } => {
                        let vertex_array = self.gl.create_vertex_array().unwrap();
                        self.gl.bind_vertex_array(Some(vertex_array));

                        let max = PI * 2.;
                        let mut vertices = Vec::with_capacity(max as usize + 1);
                        let mut i = 0.;
                        let points = r * 0.64;
                        while i < max {
                            vertices.push(i.cos() * r);
                            vertices.push(i.sin() * r);
                            vertices.push(0.);
                            i += max / points;
                        }

                        let mut vertex_buffer_data = Vec::<u8>::with_capacity(vertices.len() * 4);
                        for float in vertices.iter() {
                            vertex_buffer_data.extend_from_slice(&float.to_le_bytes());
                        }


                        let vertex_buffer = self.gl.create_buffer().unwrap();
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
                        self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_buffer_data.as_ref(), glow::DYNAMIC_DRAW);

                        self.gl.enable_vertex_attrib_array(0);
                        self.gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                        let loc = &self.gl.get_uniform_location(self.program, "transform").unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        mat.w.x = x;
                        mat.w.y = y;
                        let final_mat = self.projection * mat;
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_3_f32(Some(loc), color.r, color.g, color.b);

                        self.gl.draw_arrays(glow::TRIANGLE_FAN, 0, vertices.len() as i32);

                        self.gl.delete_vertex_array(vertex_array); // clean up
                        self.gl.delete_buffer(vertex_buffer);
                    },
                    _ => ()
                }
            }
        }
    }

    pub fn triangle (&mut self, x: f32, y: f32) {
        self.command_stack.push(Command::Triangle { x, y })
    }

    pub fn circle (&mut self, x: f32, y: f32, r: f32, color: Color) {
        self.command_stack.push(Command::Circle { x, y, r, color })
    }

    pub fn resize (&mut self, width: i32, height: i32) {
        unsafe {
            let size = 200.;
            let aspect = width as f32 / height as f32;
            self.projection = cgmath::ortho(
                0.,
                width as f32,
                height as f32,
                0.,
                0.,
                1.
            );
            self.gl.viewport(0, 0, width, height); // resize viewport
        }
    }

    pub fn destroy (self) {
        unsafe { self.gl.delete_program(self.program) }
    }
}


use std::ffi::c_void;
use std::ffi::{CStr, CString};


#[no_mangle]
pub unsafe extern "C" fn bfr_create_surface(loader: extern "C" fn(*const libc::c_char) -> *const c_void) -> *mut Renderer {
    let renderer = Box::new(Renderer::new(|s| loader(s.as_ptr() as *const libc::c_char)));
    Box::into_raw(renderer)
}

#[no_mangle]
pub unsafe extern "C" fn bfr_resize(renderer: *mut Renderer, width: i32, height: i32) {
    (*renderer).resize(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_destroy(renderer: *mut Renderer) {
    (*renderer).gl.delete_program((*renderer).program); // TODO: Fix
    libc::free(renderer as *mut libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_flush(renderer: *mut Renderer) {
    (*renderer).flush();
}

#[no_mangle]
pub unsafe extern "C" fn bfr_circle(renderer: *mut Renderer, x: f32, y: f32, r: f32, color: Color) {
    (*renderer).circle(x, y, r, color);
}
