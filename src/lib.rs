use cgmath::SquareMatrix;
use lyon::math::point;
use std::{iter, os::linux::raw};
use wgpu::util::DeviceExt;
use std::collections::HashMap;
use ordered_float::OrderedFloat;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use log::*;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn from_transform(transform: cgmath::Matrix4<f32>, width: u32, height: u32) -> Self {
        let view = transform;

        let proj = cgmath::ortho(0., width as f32, height as f32, 0., 0., 1.);
        Self {
            view_proj: (OPENGL_TO_WGPU_MATRIX * proj * view).into()
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// Represents a color with values from 0-1
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    pub r: OrderedFloat<f32>,
    pub g: OrderedFloat<f32>,
    pub b: OrderedFloat<f32>,
    pub a: OrderedFloat<f32>,
}

impl Color {
    pub fn from_f(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r: OrderedFloat(r), g: OrderedFloat(g), b: OrderedFloat(b), a: OrderedFloat(a) }
    }

    pub fn from_8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: OrderedFloat(r as f32 / 255.),
            g: OrderedFloat(g as f32 / 255.),
            b: OrderedFloat(b as f32 / 255.),
            a: OrderedFloat(a as f32 / 255.),
        }
    }

    pub fn as_array(&self) -> [f32; 4] {
        [self.r.into_inner(), self.g.into_inner(), self.b.into_inner(), self.a.into_inner()]
    }
}

struct UniformBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

enum Command {
    RawGeometry {
        path: UniqueGeometry,
        uniform_buffer: UniformBuffer,
        indices: usize,
        transform: cgmath::Matrix4<f32>,
    },
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
enum UniqueGeometry {
    Rectangle(OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>, Color),
    Circle(OrderedFloat<f32>, OrderedFloat<f32>, OrderedFloat<f32>, Color)
}

#[derive(Debug)]
struct GeometryBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: usize,
}

/// Object that manages the window and GPU resources
pub struct Painter {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    stack: Vec<Command>,

    geometry_buffers: HashMap<UniqueGeometry, GeometryBuffer>,

    transform: cgmath::Matrix4<f32>,
    old_transform: cgmath::Matrix4<f32>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    // window stuff
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl Painter {
    pub async fn new_from_window(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .unwrap();
        
        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            surface: surface,
            surface_config: config,
            device: device,
            queue: queue,
            render_pipeline,
            size: size,
            stack: Vec::new(),
            geometry_buffers: HashMap::new(),
            uniform_bind_group_layout,
            transform: cgmath::Matrix4::identity(),
            old_transform: cgmath::Matrix4::identity(),
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

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && new_size != self.size {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color,) {
        use lyon::math::{point, Point};
        use lyon::path::builder::*;
        use lyon::path::Path;
        use lyon::tessellation::*;

        let uniforms = Uniforms::from_transform(self.transform, self.size.width, self.size.height);

        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let uniform_buffer = UniformBuffer {
            buffer: uniform_buffer,
            bind_group: uniform_bind_group
        };

        let uniq = UniqueGeometry::Rectangle(OrderedFloat(x), OrderedFloat(y), OrderedFloat(width), OrderedFloat(height), color);
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_buffer: uniform_buffer,
                    path: uniq,
                });
            },
            None => {
                let mut builder = Path::builder();
                builder.begin(point(x, y));
                builder.line_to(point(width + x, y));
                builder.line_to(point(width + x, height + y));
                builder.line_to(point(x, height + y));
                builder.line_to(point(x, y));
                builder.close();
                let path = builder.build();
                let options = FillOptions::tolerance(0.1);
                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut tessellator = FillTessellator::new();
                let raw_color = color.as_array();
                let raw_color = [raw_color[0], raw_color[1], raw_color[2]];
        
                {
                    // Compute the tessellation.
                    tessellator
                        .tessellate_path(
                            &path,
                            &options,
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                                position: vertex.position().to_array(),
                                color: raw_color
                            }),
                        )
                        .unwrap();
                }
        
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&geometry.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&geometry.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                
                self.geometry_buffers.insert(uniq, GeometryBuffer {
                    vertex_buffer,
                    index_buffer,
                    indices: geometry.indices.len()
                });

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_buffer: uniform_buffer,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn circle(&mut self, x: f32, y: f32, radius: f32, color: Color,) {
        use lyon::math::{rect, Point};
        use lyon::path::{builder::*, Winding};
        use lyon::tessellation::*;
        use lyon::tessellation::geometry_builder::BuffersBuilder;

        let uniforms = Uniforms::from_transform(self.transform, self.size.width, self.size.height);

        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let uniform_buffer = UniformBuffer {
            buffer: uniform_buffer,
            bind_group: uniform_bind_group
        };

        let uniq = UniqueGeometry::Circle(OrderedFloat(x), OrderedFloat(y), OrderedFloat(radius), color);
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_buffer,
                    indices: buf.indices,
                    path: uniq,
                });
            },
            None => {
                let raw_color = color.as_array();
                let raw_color = [raw_color[0], raw_color[1], raw_color[2]];

                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut geometry_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                    position: vertex.position().to_array(),
                    color: raw_color
                });
                let options = FillOptions::tolerance(0.1);
                let mut tessellator = FillTessellator::new();

                let mut builder = tessellator.builder(
                    &options,
                    &mut geometry_builder,
                );

                builder.add_circle(
                    point(x, y),
                    radius,
                    Winding::Positive
                );

                builder.build().unwrap();
        
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&geometry.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&geometry.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                
                self.geometry_buffers.insert(uniq, GeometryBuffer {
                    vertex_buffer,
                    index_buffer,
                    indices: geometry.indices.len()
                });

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_buffer,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn flush(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_frame()?.output;
        let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });


        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            for command in self.stack.iter() {
                match command {
                    Command::RawGeometry { path, transform, indices, uniform_buffer } => {
                        render_pass.set_bind_group(0, &uniform_buffer.bind_group, &[]);

                        let buffers = self.geometry_buffers.get(&path).unwrap();

                        render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..*indices as u32, 0, 0..1);
                    }
                }
            }
        }

        
        self.queue.submit(iter::once(encoder.finish()));

        self.stack.clear();

        self.reset();

        Ok(())
    }
}
