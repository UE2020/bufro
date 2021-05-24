use cgmath::SquareMatrix;
use cgmath::Zero;

use glow::*;

pub use owned_ttf_parser::OutlineBuilder as PathBuilder;
use owned_ttf_parser::AsFaceRef;

use std::convert::AsRef;
use std::f32::consts::PI;
use std::collections::HashMap;
use std::sync::Arc;

pub mod ffi;

#[derive(Copy, Clone, Debug)]
struct PathVertex { position: [f32; 2] }

pub struct TextRenderer {
    font: owned_ttf_parser::OwnedFace,
    cache: HashMap<char, lyon::tessellation::VertexBuffers<PathVertex, u16>>,
    texture_cache: HashMap<char, u32>, // textures
    pub units_per_em: u16
}

impl TextRenderer {
    pub fn new (font: Vec<u8>) -> Option<Self> {
        let owned_face = owned_ttf_parser::OwnedFace::from_vec(font, 0).ok()?;
        let units_per_em = owned_face.as_face_ref().units_per_em()?;
        
        Some(Self {
            font: owned_face,
            cache: HashMap::new(),
            texture_cache: HashMap::new(),
            units_per_em
        })
    }

    fn create_path_from_char(&self, c: char) -> Path {
        self.create_path(self.font.as_face_ref().glyph_index(c).unwrap())
    }

    fn get_horizontal_advance(&self, c: char) -> Option<u16> {
        self.font.as_face_ref().glyph_hor_advance(self.font.as_face_ref().glyph_index(c)?)
    }

    fn get_vertical_advance(&self, c: char) -> Option<u16> {
        self.font.as_face_ref().glyph_ver_advance(self.font.as_face_ref().glyph_index(c)?)
    }

    fn create_path(&self, glyph: owned_ttf_parser::GlyphId) -> Path {
        let mut path = Path::new();
        self.font.as_face_ref().outline_glyph(glyph, &mut path);
        path
    }

    fn height(&self) -> i16 {
        self.font.as_face_ref().height()
    }

    fn line_gap(&self) -> i16 {
        self.font.as_face_ref().line_gap()
    }

    fn create_texture(&self, ctx: &Renderer, color: Color, path: lyon::lyon_tessellation::VertexBuffers<PathVertex, u16>, width: u16, height: u16) -> u32 {
        unsafe {
            let framebuffer_name = ctx.gl.create_framebuffer().unwrap();
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer_name));
            let texture = ctx.gl.create_texture().unwrap();
            ctx.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            ctx.gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, width as i32, height as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);

            ctx.gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(texture), 0);
            ctx.gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);

            ctx.gl.viewport(0, 0, width as i32, height as i32);

            ctx.gl.bind_vertex_array(Some(ctx.vertex_array));
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(ctx.vertex_buffer));
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.index_buffer));

            ctx.gl.enable_vertex_attrib_array(0);
            ctx.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
            ctx.gl.use_program(Some(ctx.program));
            
            let indices = Renderer::fill_buffers_from_raw_geometry(&ctx.gl, path);

            let loc = &ctx
                .gl
                .get_uniform_location(ctx.program, "transform")
                .unwrap();

            
            
            let mut mat = cgmath::Matrix4::identity();
            mat.w.y = height as f32 / 4.;

            let final_mat = cgmath::ortho(0., width as f32, height as f32, 0., 0., 1.) * mat * cgmath::Matrix4::from_nonuniform_scale(1., -1., 1.);
            let proj: &[f32; 16] = final_mat.as_ref();
            ctx.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

            let loc = &ctx.gl.get_uniform_location(ctx.program, "col").unwrap();
            ctx.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

            ctx.gl
                .draw_elements(glow::TRIANGLES, indices, glow::UNSIGNED_SHORT, 0);
            
            ctx.gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, None, 0);
            ctx.gl.delete_framebuffer(framebuffer_name);
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            ctx.gl.bind_texture(glow::TEXTURE_2D, None);
            ctx.gl.viewport(0, 0, ctx.width as i32, ctx.height as i32);
            texture
        }
    }

    fn get_texture(&mut self, renderer: &Renderer, c: char) -> u32 {
        match self.texture_cache.get(&c) {
            Some(texture) => *texture,
            None => {
                let tess = self.tessellate(c);
                let texture = self.create_texture(&renderer, Color::from_f(1., 1., 1., 1.), tess, self.get_horizontal_advance(c).unwrap(), (self.height() as f32/1.15) as u16);
                self.texture_cache.insert(c, texture);
                println!("Created texture! {}", texture);
                texture
            }
        }
    }

    fn tessellate(&mut self, c: char) -> lyon::tessellation::VertexBuffers<PathVertex, u16> {
        match self.cache.get(&c) {
            Some(geometry) => geometry.clone(),
            None => {
                let path = self.create_path_from_char(c);
                use lyon::tessellation::*;

                // lets build the path
                let path = path.ctx.build();
                let mut geometry: lyon::tessellation::VertexBuffers<PathVertex, u16> = lyon::tessellation::VertexBuffers::new();
                let mut tessellator = lyon::tessellation::FillTessellator::new();
        
                {
                    // Compute the tessellation.
                    tessellator.tessellate_path(
                        &path,
                        &FillOptions::default().with_tolerance(4.),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                            PathVertex {
                                position: vertex.position().to_array(),
                            }
                        }),
                    ).unwrap();
                }
                self.cache.insert(c, geometry.clone());
                geometry
            }
        }
    }
}

pub struct Path {
    ctx: lyon::path::path::Builder,
}

impl Path {
    pub fn new () -> Self {
        let builder = lyon::path::Path::builder();
        Self {
            ctx: builder
        }
    }

    pub fn close (&mut self) {
        self.ctx.close();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.ctx.begin(lyon::math::point(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.ctx.line_to(lyon::math::point(x, y));
    }
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.ctx.quadratic_bezier_to(lyon::math::point(x1, y1), lyon::math::point(x, y));
    }
    pub fn curve_to(
        &mut self, 
        x1: f32, 
        y1: f32, 
        x2: f32, 
        y2: f32, 
        x: f32, 
        y: f32
    ) {
        self.ctx.cubic_bezier_to(lyon::math::point(x1, y1), lyon::math::point(x2, y2), lyon::math::point(x, y));
    }
}

impl owned_ttf_parser::OutlineBuilder for Path {
    fn close (&mut self) {
        self.ctx.close();
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        self.ctx.begin(lyon::math::point(x, -y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.ctx.line_to(lyon::math::point(x, -y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.ctx.quadratic_bezier_to(lyon::math::point(x1, -y1), lyon::math::point(x, -y));
    }
    fn curve_to(
        &mut self, 
        x1: f32, 
        y1: f32, 
        x2: f32, 
        y2: f32, 
        x: f32, 
        y: f32
    ) {
        self.ctx.cubic_bezier_to(lyon::math::point(x1, -y1), lyon::math::point(x2, -y2), lyon::math::point(x, -y));
    }
}

fn compile_shader (gl: &glow::Context, vertex_shader_source: &str, fragment_shader_source: &str) -> u32 {
    unsafe {
        let program = gl.create_program().expect("Cannot create program"); // compile and link shader program

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

        program
    }
}

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

#[repr(C)]
pub enum GeometryStyle {
    Fill,
    Stroke
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
    },
    Geometry {
        color: Color,
        style: GeometryStyle,
        path: Path,
        transform: cgmath::Matrix4<f32>
    },
    RawGeometry {
        color: Color,
        path: lyon::tessellation::VertexBuffers<PathVertex, u16>,
        transform: cgmath::Matrix4<f32>
    },
    Texture {
        name: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        angle: f32,
        transform: cgmath::Matrix4<f32>
    }
}

/// A basic window-less renderer (though you can always just load the function pointers of a window)
pub struct Renderer {
    gl: Context,                 // the main context
    program: u32,                // the main shader
    texture_program: u32,        // shader used to render textures
    command_stack: Vec<Command>, // clear command_stack on flush,
    projection: cgmath::Matrix4<f32>,

    // caching
    vertex_array: u32,
    vertex_buffer: u32,
    index_buffer: u32,

    // matrix
    transform: cgmath::Matrix4<f32>,
    old_transform: cgmath::Matrix4<f32>,

    width: u32,
    height: u32
}

impl Renderer {
    pub fn new<F: FnMut(&str) -> *const std::os::raw::c_void>(function_loader: F) -> Self {
        unsafe {
            let gl = glow::Context::from_loader_function(function_loader); // load function pointers

            let program = compile_shader(&gl, 
                r#"layout(location = 0) in vec2 vertexPosition_modelspace;
                //out vec2 vert;
                uniform mat4 transform;

                void main() {
                    gl_Position = transform * vec4(vertexPosition_modelspace, 0.0, 1.0);
                    //gl_Position.xy += pos;
                    gl_Position.w = 1.0;
                }"#,
                r#"
                layout(location = 0) out vec4 color;
                uniform vec4 col;

                void main(){
                    color = col;
                }"#,
            );

            let texture_program = compile_shader(&gl, 
                r#"layout(location = 0) in vec2 vertexPosition_modelspace;
                out vec2 vert;
                uniform mat4 transform;

                void main() {
                    gl_Position = transform * vec4(vertexPosition_modelspace, 0.0, 1.0);
                    vert = vertexPosition_modelspace;
                    //gl_Position.xy += pos;
                    gl_Position.w = 1.0;
                }"#,
                r#"
                layout(location = 0) out vec4 color;
                uniform sampler2D col;

                uniform vec2 imgSize;

                in vec2 vert;
                void main(){
                    color = texture(col, vec2(vert.x / imgSize.x, vert.y / imgSize.y));
                }"#,
            );

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
                texture_program,
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

                width: m_viewport[2] as u32,
                height: m_viewport[3] as u32
            }
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));

            self.gl.enable_vertex_attrib_array(0);
            self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
            self.gl.use_program(Some(self.program));
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
                        self.gl.use_program(Some(self.program));

                        let max = PI * 2.;
                        let mut vertices = Vec::with_capacity(max as usize + 1);
                        let mut i = 0.;
                        let points_calculation = r * 0.64;
                        let points = if points_calculation > 32. { points_calculation } else { 32. };
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
                        self.gl.use_program(Some(self.program));

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
                        self.gl.use_program(Some(self.program));

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
                    },
                    Command::Texture {
                        x,
                        y,
                        width,
                        height,
                        name,
                        angle,
                        transform,
                    } => {
                        self.gl.use_program(Some(self.texture_program));
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(name));

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
                            .get_uniform_location(self.texture_program, "transform")
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

                        let loc = &self.gl.get_uniform_location(self.texture_program, "col").unwrap();
                        self.gl.uniform_1_u32(Some(loc), name);

                        let loc = &self.gl.get_uniform_location(self.texture_program, "imgSize").unwrap();
                        self.gl.uniform_2_f32(Some(loc), width, height);

                        self.gl
                            .draw_arrays(glow::TRIANGLES, 0, vertex_buffer_data.len() as i32);

                    },
                    Command::Geometry {
                        color,
                        transform,
                        path,
                        style
                    } => {
                        self.gl.use_program(Some(self.program));

                        use lyon::tessellation::*;

                        // lets build the path
                        let path = path.ctx.build();
                        let mut geometry: lyon::tessellation::VertexBuffers<PathVertex, u16> = lyon::tessellation::VertexBuffers::new();
                        let mut tessellator = lyon::tessellation::FillTessellator::new();

                        {
                            // Compute the tessellation.
                            tessellator.tessellate_path(
                                &path,
                                &FillOptions::default(),
                                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                                    PathVertex {
                                        position: vertex.position().to_array(),
                                    }
                                }),
                            ).unwrap();
                        }
                        let indices = Self::fill_buffers_from_raw_geometry(&self.gl, geometry);

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        let final_mat = self.projection * transform * mat; // * cgmath::Matrix4::from_nonuniform_scale(1., -1., 1.);
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

                        self.gl
                            .draw_elements(glow::TRIANGLES, indices, glow::UNSIGNED_SHORT, 0);
                    },
                    Command::RawGeometry {
                        color,
                        transform,
                        path
                    } => {
                        self.gl.use_program(Some(self.program));

                        let indices = Self::fill_buffers_from_raw_geometry(&self.gl, path);

                        let loc = &self
                            .gl
                            .get_uniform_location(self.program, "transform")
                            .unwrap();

                        let mut mat = cgmath::Matrix4::identity();
                        let final_mat = self.projection * transform * mat; // * cgmath::Matrix4::from_nonuniform_scale(1., -1., 1.);
                        let proj: &[f32; 16] = final_mat.as_ref();
                        self.gl.uniform_matrix_4_f32_slice(Some(loc), false, proj);

                        let loc = &self.gl.get_uniform_location(self.program, "col").unwrap();
                        self.gl.uniform_4_f32(Some(loc), color.r, color.g, color.b, color.a);

                        self.gl
                            .draw_elements(glow::TRIANGLES, indices, glow::UNSIGNED_SHORT, 0);
                    }
                }
            }

            self.transform = cgmath::Matrix4::identity();
        }
    }

    fn fill_buffers_from_raw_geometry(gl: &glow::Context, geometry: lyon::tessellation::VertexBuffers<PathVertex, u16>) -> i32 {
        unsafe {
            // SAFETY: Assume all buffers are bound
            let mut vertex_buffer_data = vec![];
            for vertex in geometry.vertices {
                vertex_buffer_data.push(vertex.position[0]);
                vertex_buffer_data.push(vertex.position[1]);
            }


            let mut vertices = Vec::<u8>::with_capacity(vertex_buffer_data.len() * 4);
            for float in vertex_buffer_data.iter() {
                vertices.extend_from_slice(&float.to_le_bytes());
            }

            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                vertices.as_ref(),
                glow::DYNAMIC_DRAW,
            );

            let mut indices = Vec::<u8>::with_capacity(geometry.indices.len() * 2);
            for float in geometry.indices.iter() {
                indices.extend_from_slice(&float.to_le_bytes());
            }

            // index buffer
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices.as_ref(), glow::DYNAMIC_DRAW);

            geometry.indices.len() as i32
        }
    }

    pub fn triangle(&mut self, x: f32, y: f32) {
        self.command_stack.push(Command::Triangle { x, y })
    }

    pub fn texture(&mut self, x: f32, y: f32, width: f32, height: f32, angle: f32, name: u32) {
        self.command_stack.push(Command::Texture {
            x,
            y,
            width,
            height,
            angle,
            name,
            transform: self.transform.clone(),
        })
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

    pub fn text(&mut self, text: &str, color: Color, font: &mut TextRenderer, x: f32, y: f32, scale: f32, line_break_length: f32) {
        let old_transform = self.transform.clone();
        self.translate(x, y);
        let scale_factor = scale / font.units_per_em as f32;
        self.scale(scale_factor, scale_factor);
        self.translate(0., font.height() as f32/1.5);
        let mut offset = 0.;
        for glyph in text.chars() {
            if offset * scale_factor >= line_break_length {
                self.translate(-offset, font.height() as f32/1.15);
                offset = 0.;
            }
            if glyph == '\n' {
                self.translate(-offset, font.height() as f32/1.15);
                offset = 0.;
                continue;
            }
            
            let tex = font.get_texture(&self, glyph);
            self.texture(0., 0., font.get_horizontal_advance(glyph).unwrap() as f32, font.height() as f32/1.15, 0., tex);

            let advance = font.get_horizontal_advance(glyph).unwrap() as f32;
            offset += advance;
            self.translate(advance, 0.);
        }
        self.transform = old_transform;
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

    pub fn path(&mut self, path: Path, style: GeometryStyle, color: Color) {
        self.command_stack.push(Command::Geometry {
            color,
            path,
            transform: self.transform.clone(),
            style
        })
    }

    fn raw_geometry(&mut self, path: lyon::tessellation::VertexBuffers<PathVertex, u16>, color: Color) {
        self.command_stack.push(Command::RawGeometry {
            color,
            path,
            transform: self.transform.clone(),
        })
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.width = width as u32;
            self.height = height as u32;
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
