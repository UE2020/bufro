use cgmath::SquareMatrix;
use lyon::math::point;
use ordered_float::OrderedFloat;
use owned_ttf_parser::AsFaceRef;
use std::collections::HashMap;
use std::iter;
use wgpu::util::DeviceExt;
use wgpu_profiler::*;

use winit::window::Window;
use cgmath::Transform;

#[allow(unused_imports)]
use log::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],
    _padding: [f32; 48],
}

impl Uniforms {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            _padding: [0.0; 48],
        }
    }

    fn from_transform(transform: cgmath::Matrix4<f32>, width: u32, height: u32) -> Self {
        let view = transform;

        let proj = cgmath::ortho(0., width as f32, height as f32, 0., 0., 1.);
        Self {
            view_proj: (OPENGL_TO_WGPU_MATRIX * proj * view).into(),
            _padding: [0.0; 48],
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
        Self {
            r: OrderedFloat(r),
            g: OrderedFloat(g),
            b: OrderedFloat(b),
            a: OrderedFloat(a),
        }
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
        [
            self.r.into_inner(),
            self.g.into_inner(),
            self.b.into_inner(),
            self.a.into_inner(),
        ]
    }
}

struct UniformBuffer {
    #[allow(dead_code)]
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
    mem_align: mem_align::MemAlign<Uniforms>,
}

impl UniformBuffer {
    pub fn new(
        device: &wgpu::Device,
        capacity: usize,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mem_align: mem_align::MemAlign<Uniforms> = mem_align::MemAlign::new(capacity);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            size: mem_align.byte_size() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(wgpu::BufferSize::new(256).unwrap()),
                }),
            }],
            label: Some("Uniform Bind Group"),
        });

        Self {
            buffer: uniform_buffer,
            bind_group: uniform_bind_group,
            capacity: mem_align.capacity(),
            mem_align: mem_align,
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        capacity: usize,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        if capacity <= self.capacity {
            return;
        }

        let mem_align = mem_align::MemAlign::<Uniforms>::new(capacity);

        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            size: mem_align.byte_size() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &inner,
                    offset: 0,
                    size: Some(wgpu::BufferSize::new(256).unwrap()),
                }),
            }],
            label: Some("Uniform Bind Group"),
        });

        self.mem_align = mem_align;
        self.buffer = inner;
        self.bind_group = uniform_bind_group;
    }
}

enum Command {
    RawGeometry {
        path: UniqueGeometry,
        uniform_offset: wgpu::BufferAddress,
        indices: usize,

        #[allow(dead_code)]
        transform: cgmath::Matrix4<f32>,
    },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum UniqueGeometry {
    Rectangle(
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        Color,
    ),
    Circle(
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        Color,
    ),
    Path(Vec<PathInstruction>, cgmath::Matrix4<OrderedFloat<f32>>,  Color),
}

#[derive(Debug)]
struct GeometryBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: usize,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct HashablePoint(OrderedFloat<f32>, OrderedFloat<f32>);

impl HashablePoint {
    fn new(x: f32, y: f32) -> Self {
        Self(OrderedFloat(x), OrderedFloat(y))
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum PathInstruction {
    MoveTo(HashablePoint),
    Close,
    LineTo(HashablePoint),
    QuadTo(HashablePoint, HashablePoint),
    CurveTo(HashablePoint, HashablePoint, HashablePoint),
    Arc(
        HashablePoint,
        HashablePoint,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
    ),
}

struct TransformableMatrix(cgmath::Matrix4<f32>);

impl lyon::geom::traits::Transformation<f32> for TransformableMatrix {
    fn transform_point(&self, p: lyon::geom::Point<f32>) -> lyon::geom::Point<f32> {
        let res = self.0.transform_point(cgmath::point3(p.x, p.y, 0.0));
        lyon::geom::point(res.x, res.y)
    }

    fn transform_vector(&self, v: lyon::geom::Vector<f32>) -> lyon::geom::Vector<f32> {
        let res = self.0.transform_vector(cgmath::vec3(v.x, v.y, 0.0));
        lyon::geom::vector(res.x, res.y)
    }
}

impl Into<TransformableMatrix> for cgmath::Matrix4<f32> {
    fn into(self) -> TransformableMatrix {
        TransformableMatrix(self)
    }
}

// Builds a geometry buffer from a path
pub struct PathBuilder {
    path: lyon::path::builder::WithSvg<lyon::path::builder::Transformed<lyon::path::path::Builder, TransformableMatrix>>,
    path_instructions: Vec<PathInstruction>,
    transform: cgmath::Matrix4<f32>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            path: lyon::path::Path::builder().with_svg().transformed(cgmath::Matrix4::identity().into()),
            path_instructions: Vec::new(),
            transform: cgmath::Matrix4::identity(),
        }
    }

    pub fn new_with_transform(transform: cgmath::Matrix4<f32>) -> Self {
        Self {
            path: lyon::path::Path::builder().with_svg().transformed(transform.into()),
            path_instructions: Vec::new(),
            transform: transform,
        }
    }

    /// Closes the path
    pub fn close(&mut self) {
        self.path.close();
        self.path_instructions.push(PathInstruction::Close);
    }

    /// Moves the current point to the given point
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(lyon::math::point(x, y));
        self.path_instructions
            .push(PathInstruction::MoveTo(HashablePoint::new(x, y)));
    }

    /// Adds line to path
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(lyon::math::point(x, y));
        self.path_instructions
            .push(PathInstruction::LineTo(HashablePoint::new(x, y)));
    }

    /// Adds quadriatic bezier to path
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path
            .quadratic_bezier_to(lyon::math::point(x1, y1), lyon::math::point(x, y));
        self.path_instructions.push(PathInstruction::QuadTo(
            HashablePoint::new(x1, y1),
            HashablePoint::new(x, y),
        ));
    }

    /// Adds arc to path
    pub fn arc(&mut self, x: f32, y: f32, x1: f32, y1: f32, sweep_angle: f32, x_rotation: f32) {
        self.path.arc(
            lyon::math::point(x, y),
            lyon::math::vector(x, y),
            lyon::math::Angle::radians(sweep_angle),
            lyon::math::Angle::radians(x_rotation),
        );
        self.path_instructions.push(PathInstruction::Arc(
            HashablePoint::new(x1, y1),
            HashablePoint::new(x, y),
            OrderedFloat(sweep_angle),
            OrderedFloat(x_rotation),
        ));
    }

    /// Adds cubic bezier to path
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.cubic_bezier_to(
            lyon::math::point(x1, y1),
            lyon::math::point(x2, y2),
            lyon::math::point(x, y),
        );
        self.path_instructions.push(PathInstruction::CurveTo(
            HashablePoint::new(x1, y1),
            HashablePoint::new(x2, y2),
            HashablePoint::new(x, y),
        ));
    }
}

pub struct Font {
    font: owned_ttf_parser::OwnedFace,
}

impl Font {
    pub fn new(font: &[u8]) -> Option<Self> {
        let owned_face = owned_ttf_parser::OwnedFace::from_vec(font.to_vec(), 0).ok()?;

        Some(Self { font: owned_face })
    }
}

impl owned_ttf_parser::OutlineBuilder for PathBuilder {
    fn close(&mut self) {
        self.close();
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(x, -y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(x, -y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.quad_to(x1, -y1, x, -y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.curve_to(x1, -y1, x2, -y2, x, -y);
    }
}

// Represents a drawable surface
pub struct Surface {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

/// Object that manages the window and GPU resources
pub struct Painter {
    surface: Surface,
    multisampled_framebuffer: wgpu::TextureView,

    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    stack: Vec<Command>,

    geometry_buffers: HashMap<UniqueGeometry, GeometryBuffer>,

    transform: cgmath::Matrix4<f32>,
    old_transform: cgmath::Matrix4<f32>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    clear_color: Color,
    uniform_vec: Vec<Uniforms>,
    uniform_buffer: UniformBuffer,

    profiler: GpuProfiler,
    pub latest_profiler_results: Option<Vec<GpuTimerScopeResult>>,
}

impl Painter {
    pub async fn new_from_window(window: &Window) -> Self {
        let size = window.inner_size();

        println!("Params {}", std::mem::size_of::<Uniforms>());

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
                    features: wgpu::Features::TIMESTAMP_QUERY,
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .unwrap();

        let swapchain_format = wgpu::TextureFormat::Bgra8Unorm;

        let config = wgpu::SurfaceConfiguration {
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
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Uniforms>() as _
                        ),
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
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
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
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let multisampled_framebuffer = Self::create_multisampled_framebuffer(&device, &config, 4);

        let uniform_buffer = UniformBuffer::new(&device, 100, &uniform_bind_group_layout);

        let mut profiler = GpuProfiler::new(4, queue.get_timestamp_period()); // buffer up to 4 frames

        Self {
            surface: Surface {
                surface,
                surface_config: config,
                size: size,
            },
            device: device,
            queue: queue,
            render_pipeline,
            stack: Vec::new(),
            geometry_buffers: HashMap::new(),
            uniform_bind_group_layout,
            transform: cgmath::Matrix4::identity(),
            old_transform: cgmath::Matrix4::identity(),
            clear_color: Color::from_8(0, 0, 0, 0xff),
            multisampled_framebuffer,
            uniform_vec: Vec::new(),
            uniform_buffer: uniform_buffer,
            profiler,
            latest_profiler_results: None,
        }
    }

    fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
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

    pub fn fill_text(&mut self, font: &Font, text: &str, x: f32, y: f32, color: Color) {
        let face = font.font.as_face_ref();
        let old_transform = self.transform;
        self.translate(x, 50.);
        self.scale(0.005, 0.005);
        let mut offset = 0.;

        let mut path = PathBuilder::new();
        let glyph = face.as_face_ref().glyph_index('A').unwrap();
        face.outline_glyph(glyph, &mut path);

        for character in text.chars() {
            if character == '\n' {
                self.translate(-offset, 50./0.005);
                offset = 0.;
                continue;
            }
            
            //self.fill_path(&path, color);
            self.circle(0., 0., 5000., color);
            offset += face.glyph_hor_advance(glyph).unwrap() as f32;
            self.translate(face.glyph_hor_advance(glyph).unwrap() as f32, 0.);
        }
        self.transform = old_transform;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && new_size != self.surface.size {
            self.surface.size = new_size;
            self.surface.surface_config.width = new_size.width;
            self.surface.surface_config.height = new_size.height;
            self.multisampled_framebuffer = Self::create_multisampled_framebuffer(
                &self.device,
                &self.surface.surface_config,
                4,
            );
            self.surface
                .surface
                .configure(&self.device, &self.surface.surface_config);
        }
    }

    pub fn regen(&mut self) {
        self.surface
            .surface
            .configure(&self.device, &self.surface.surface_config);
    }

    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        use lyon::path::Path;
        use lyon::tessellation::*;

        let uniforms = Uniforms::from_transform(
            self.transform,
            self.surface.size.width,
            self.surface.size.height,
        );

        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Rectangle(
            OrderedFloat(x),
            OrderedFloat(y),
            OrderedFloat(width),
            OrderedFloat(height),
            color,
        );
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    path: uniq,
                });
            }
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

                {
                    // Compute the tessellation.
                    tessellator
                        .tessellate_path(
                            &path,
                            &options,
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                                position: vertex.position().to_array(),
                                color: raw_color,
                            }),
                        )
                        .unwrap();
                }

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&geometry.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&geometry.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                self.geometry_buffers.insert(
                    uniq.clone(),
                    GeometryBuffer {
                        vertex_buffer,
                        index_buffer,
                        indices: geometry.indices.len(),
                    },
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn stroke_path(&mut self, path_builder: PathBuilder, color: Color) {
        use lyon::tessellation::*;

        let uniforms = Uniforms::from_transform(
            self.transform,
            self.surface.size.width,
            self.surface.size.height,
        );

        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Path(path_builder.path_instructions, color);
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    path: uniq,
                });
            }
            None => {
                let path = path_builder.path.build();
                let options = StrokeOptions::tolerance(0.1).with_line_width(20.);
                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut tessellator = StrokeTessellator::new();
                let raw_color = color.as_array();

                {
                    // Compute the tessellation.
                    tessellator
                        .tessellate_path(
                            &path,
                            &options,
                            &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                                Vertex {
                                    position: vertex.position().to_array(),
                                    color: raw_color,
                                }
                            }),
                        )
                        .unwrap();
                }

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&geometry.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&geometry.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                self.geometry_buffers.insert(
                    uniq.clone(),
                    GeometryBuffer {
                        vertex_buffer,
                        index_buffer,
                        indices: geometry.indices.len(),
                    },
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn fill_path(&mut self, path_builder: PathBuilder, color: Color) {
        use lyon::tessellation::*;

        let uniforms = Uniforms::from_transform(
            self.transform,
            self.surface.size.width,
            self.surface.size.height,
        );
        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Path(path_builder.path_instructions, color);
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    path: uniq,
                });
            }
            None => {
                let path = path_builder.path.build();
                let options = FillOptions::tolerance(1.0);
                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut tessellator = FillTessellator::new();
                let raw_color = color.as_array();

                {
                    // Compute the tessellation.
                    tessellator
                        .tessellate_path(
                            &path,
                            &options,
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                                position: vertex.position().to_array(),
                                color: raw_color,
                            }),
                        )
                        .unwrap();
                }

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&geometry.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&geometry.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                self.geometry_buffers.insert(
                    uniq.clone(),
                    GeometryBuffer {
                        vertex_buffer,
                        index_buffer,
                        indices: geometry.indices.len(),
                    },
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        use lyon::path::{builder::*, Winding};
        use lyon::tessellation::geometry_builder::BuffersBuilder;
        use lyon::tessellation::*;

        let uniforms = Uniforms::from_transform(
            self.transform,
            self.surface.size.width,
            self.surface.size.height,
        );
        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Circle(
            OrderedFloat(x),
            OrderedFloat(y),
            OrderedFloat(radius),
            color,
        );
        match self.geometry_buffers.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: buf.indices,
                    path: uniq,
                });
            }
            None => {
                let raw_color = color.as_array();

                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut geometry_builder =
                    BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                        position: vertex.position().to_array(),
                        color: raw_color,
                    });
                let options = FillOptions::tolerance(0.1);
                let mut tessellator = FillTessellator::new();

                let mut builder = tessellator.builder(&options, &mut geometry_builder);

                builder.add_circle(point(x, y), radius, Winding::Positive);

                builder.build().unwrap();

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&geometry.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&geometry.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                self.geometry_buffers.insert(
                    uniq.clone(),
                    GeometryBuffer {
                        vertex_buffer,
                        index_buffer,
                        indices: geometry.indices.len(),
                    },
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                });
            }
        }
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.uniform_vec.clear();

        self.reset();
    }

    pub fn flush(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.surface.get_current_frame()?.output;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.uniform_buffer.resize(
            &self.device,
            self.uniform_vec.len(),
            &self.uniform_bind_group_layout,
        );
        self.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&self.uniform_vec),
        );

        {
            wgpu_profiler!("Render Geometry", &mut self.profiler, &mut encoder, &self.device, {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &self.multisampled_framebuffer,
                        resolve_target: Some(&view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.clear_color.r.into_inner() as f64,
                                g: self.clear_color.g.into_inner() as f64,
                                b: self.clear_color.b.into_inner() as f64,
                                a: self.clear_color.a.into_inner() as f64,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                for (i, command) in self.stack.iter().enumerate() {
                    wgpu_profiler!(format!("command {}", i).as_str(), &mut self.profiler, &mut render_pass, &self.device, {
                        match command {
                            Command::RawGeometry {
                                path,
                                transform: _,
                                indices,
                                uniform_offset,
                            } => {

                                render_pass.set_bind_group(
                                    0,
                                    &self.uniform_buffer.bind_group,
                                    &[*uniform_offset as u32 * 256 as u32],
                                );
    
                                let buffers = self.geometry_buffers.get(&path).unwrap();
    
                                render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
                                render_pass.set_index_buffer(
                                    buffers.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                render_pass.draw_indexed(0..*indices as u32, 0, 0..1);
                            }
                        }
                    });
                }
            });
        }

        self.profiler.resolve_queries(&mut encoder);

        self.queue.submit(iter::once(encoder.finish()));

        self.profiler.end_frame().unwrap();

        self.stack.clear();
        self.uniform_vec.clear();

        self.reset();

        if let Some(results) = self.profiler.process_finished_frame() {
            self.latest_profiler_results = Some(results);
        }
        console_output(&self.latest_profiler_results);
        

        Ok(())
    }
}

fn scopes_to_console_recursive(results: &[GpuTimerScopeResult], indentation: u32) {
    for scope in results {
        if indentation > 0 {
            print!("{:<width$}", "|", width = 4);
        }
        println!("{:.3}μs - {}", (scope.time.end - scope.time.start) * 1000.0 * 1000.0, scope.label);
        if !scope.nested_scopes.is_empty() {
            scopes_to_console_recursive(&scope.nested_scopes, indentation + 1);
        }
    }
}

fn console_output(results: &Option<Vec<GpuTimerScopeResult>>) {
    print!("\x1B[2J\x1B[1;1H"); // Clear terminal and put cursor to first row first column
    println!("Welcome to wgpu_profiler demo!");
    println!("Press space to write out a trace file that can be viewed in chrome's chrome://tracing");
    println!();
    match results {
        Some(results) => {
            scopes_to_console_recursive(&results, 0);
        }
        None => println!("No profiling results available yet!"),
    }
}