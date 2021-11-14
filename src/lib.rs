use cgmath::SquareMatrix;
use lyon::math::point;
use ordered_float::OrderedFloat;
use owned_ttf_parser::AsFaceRef;
use std::collections::{HashMap, HashSet};
use std::iter;
use wgpu::util::DeviceExt;
use wgpu_profiler::*;

use cgmath::Transform;
use winit::window::Window;

use std::sync::Arc;

pub mod ffi;

//pub use lyon::tessellation::StrokeOptions;
pub use lyon::tessellation::FillOptions;

pub use wgpu::SurfaceError;

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
#[repr(C)]
pub enum LineCap {
    /// The stroke for each sub-path does not extend beyond its two endpoints.
    /// A zero length sub-path will therefore not have any stroke.
    Butt,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a rectangle with the same width as the stroke width and
    /// whose length is half of the stroke width. If a sub-path has zero length,
    /// then the resulting effect is that the stroke for that sub-path consists
    /// solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    Square,
    /// At each end of each sub-path, the shape representing the stroke will be extended
    /// by a half circle with a radius equal to the stroke width.
    /// If a sub-path has zero length, then the resulting effect is that the stroke for
    /// that sub-path consists solely of a full circle centered at the sub-path's point.
    Round,
}

impl Into<lyon::tessellation::LineCap> for LineCap {
    fn into(self) -> lyon::tessellation::LineCap {
        match self {
            LineCap::Butt => lyon::tessellation::LineCap::Butt,
            LineCap::Square => lyon::tessellation::LineCap::Square,
            LineCap::Round => lyon::tessellation::LineCap::Round,
        }
    }
}

/// Line join as defined by the SVG specification.
///
/// See: <https://svgwg.org/specs/strokes/#StrokeLinejoinProperty>
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
#[repr(C)]
pub enum LineJoin {
    /// A sharp corner is to be used to join path segments.
    Miter,
    /// Same as a miter join, but if the miter limit is exceeded,
    /// the miter is clipped at a miter length equal to the miter limit value
    /// multiplied by the stroke width.
    MiterClip,
    /// A round corner is to be used to join path segments.
    Round,
    /// A bevelled corner is to be used to join path segments.
    /// The bevel shape is a triangle that fills the area between the two stroked
    /// segments.
    Bevel,
}

impl Into<lyon::tessellation::LineJoin> for LineJoin {
    fn into(self) -> lyon::tessellation::LineJoin {
        match self {
            LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
            LineJoin::MiterClip => lyon::tessellation::LineJoin::MiterClip,
            LineJoin::Round => lyon::tessellation::LineJoin::Round,
            LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct StrokeOptions {
    /// What cap to use at the start of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub start_cap: LineCap,

    /// What cap to use at the end of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub end_cap: LineCap,

    /// See the SVG specification.
    ///
    /// Default value: `LineJoin::Miter`.
    pub line_join: LineJoin,

    /// Line width
    ///
    /// Default value: `StrokeOptions::DEFAULT_LINE_WIDTH`.
    pub line_width: OrderedFloat<f32>,

    /// See the SVG specification.
    ///
    /// Must be greater than or equal to 1.0.
    /// Default value: `StrokeOptions::DEFAULT_MITER_LIMIT`.
    pub miter_limit: OrderedFloat<f32>,
}

impl StrokeOptions {
    /// Minimum miter limit as defined by the SVG specification.
    ///
    /// See [StrokeMiterLimitProperty](https://svgwg.org/specs/strokes/#StrokeMiterlimitProperty)
    pub const MINIMUM_MITER_LIMIT: OrderedFloat<f32> = OrderedFloat(1.0);
    /// Default miter limit as defined by the SVG specification.
    ///
    /// See [StrokeMiterLimitProperty](https://svgwg.org/specs/strokes/#StrokeMiterlimitProperty)
    pub const DEFAULT_MITER_LIMIT: OrderedFloat<f32> = OrderedFloat(4.0);
    pub const DEFAULT_LINE_CAP: LineCap = LineCap::Butt;
    pub const DEFAULT_LINE_JOIN: LineJoin = LineJoin::Miter;
    pub const DEFAULT_LINE_WIDTH: OrderedFloat<f32> = OrderedFloat(1.0);

    pub const DEFAULT: Self = StrokeOptions {
        start_cap: Self::DEFAULT_LINE_CAP,
        end_cap: Self::DEFAULT_LINE_CAP,
        line_join: Self::DEFAULT_LINE_JOIN,
        line_width: Self::DEFAULT_LINE_WIDTH,
        miter_limit: Self::DEFAULT_MITER_LIMIT,
    };
}

impl StrokeOptions {
    #[inline]
    pub fn with_line_cap(mut self, cap: LineCap) -> Self {
        self.start_cap = cap;
        self.end_cap = cap;
        self
    }

    #[inline]
    pub fn with_start_cap(mut self, cap: LineCap) -> Self {
        self.start_cap = cap;
        self
    }

    #[inline]
    pub fn with_end_cap(mut self, cap: LineCap) -> Self {
        self.end_cap = cap;
        self
    }

    #[inline]
    pub fn with_line_join(mut self, join: LineJoin) -> Self {
        self.line_join = join;
        self
    }

    #[inline]
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = OrderedFloat(width);
        self
    }

    #[inline]
    pub fn with_miter_limit(mut self, limit: f32) -> Self {
        assert!(OrderedFloat(limit) >= Self::MINIMUM_MITER_LIMIT);
        self.miter_limit = OrderedFloat(limit);
        self
    }
}

impl Default for StrokeOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Into<lyon::tessellation::StrokeOptions> for StrokeOptions {
    fn into(self) -> lyon::tessellation::StrokeOptions {
        lyon::tessellation::StrokeOptions::tolerance(0.1)
            .with_start_cap(self.start_cap.into())
            .with_end_cap(self.end_cap.into())
            .with_line_join(self.line_join.into())
            .with_line_width(self.line_width.into_inner())
            .with_miter_limit(self.miter_limit.into_inner())
    }
}

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
#[repr(C)]
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
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
    mem_align: mem_align::MemAlign<Uniforms>,
}

impl UniformBuffer {
    fn new(
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
            mem_align: mem_align,
        }
    }

    fn capacity(&self) -> usize {
        self.mem_align.capacity()
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        capacity: usize,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        if capacity <= self.capacity() {
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
        vertices: usize,

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
    StrokedPath(Arc<Vec<PathInstruction>>, Color, StrokeOptions),
    Path(Arc<Vec<PathInstruction>>, Color),
}

#[derive(Debug)]
struct GeometryBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: usize,
    vertices: usize,
    vertex_capacity: usize,
    index_capacity: usize,
}

impl GeometryBuffer {
    fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            index_buffer,
            indices: indices.len(),
            vertices: vertices.len(),
            vertex_capacity: vertices.len(),
            index_capacity: indices.len(),
        }
    }

    fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
        indices: &[u16],
    ) {
        self.resize_vertex(device, vertices.len());
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        self.vertices = vertices.len();

        self.resize_index(device, indices.len());
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
        self.indices = indices.len();
    }

    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn index_capacity(&self) -> usize {
        self.index_capacity
    }

    fn resize_vertex(&mut self, device: &wgpu::Device, capacity: usize) {
        if capacity <= self.vertex_capacity() {
            return;
        }

        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: (capacity * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.vertex_capacity = capacity;
        self.vertex_buffer = inner;
    }

    fn resize_index(&mut self, device: &wgpu::Device, capacity: usize) {
        if capacity <= self.index_capacity() {
            return;
        }

        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index buffer"),
            size: (capacity * 2) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.index_capacity = capacity;
        self.index_buffer = inner;
    }
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
}

#[derive(Clone)]
pub struct Path {
    path: lyon::path::path::Builder,
    path_instructions: Arc<Vec<PathInstruction>>,
}

// Builds a geometry buffer from a path
#[derive(Clone)]
pub struct PathBuilder {
    path: lyon::path::path::Builder,
    path_instructions: Vec<PathInstruction>,
    transform: cgmath::Matrix4<f32>,
    old_transforms: Vec<cgmath::Matrix4<f32>>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            path: lyon::path::Path::builder(),
            path_instructions: Vec::new(),
            transform: cgmath::Matrix4::identity(),
            old_transforms: Vec::new(),
        }
    }

    pub fn build(self) -> Path {
        Path {
            path: self.path,
            path_instructions: Arc::new(self.path_instructions),
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
        self.old_transforms.push(self.transform);
    }

    pub fn restore(&mut self) {
        self.transform = self.old_transforms.pop().unwrap();
    }

    pub fn reset(&mut self) {
        self.transform = cgmath::Matrix4::identity();
    }

    /// Closes the path
    pub fn close(&mut self) {
        self.path.close();
        self.path_instructions.push(PathInstruction::Close);
    }

    /// Moves the current point to the given point
    pub fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = {
            let point = cgmath::point3(x, y, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        self.path.close();
        self.path.begin(lyon::math::point(x, y));
        self.path_instructions
            .push(PathInstruction::MoveTo(HashablePoint::new(x, y)));
    }

    /// Adds line to path
    pub fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = {
            let point = cgmath::point3(x, y, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        self.path.line_to(lyon::math::point(x, y));
        self.path_instructions
            .push(PathInstruction::LineTo(HashablePoint::new(x, y)));
    }

    /// Adds quadriatic bezier to path
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x, y) = {
            let point = cgmath::point3(x, y, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        let (x1, y1) = {
            let point = cgmath::point3(x1, y1, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        self.path
            .quadratic_bezier_to(lyon::math::point(x1, y1), lyon::math::point(x, y));
        self.path_instructions.push(PathInstruction::QuadTo(
            HashablePoint::new(x1, y1),
            HashablePoint::new(x, y),
        ));
    }

    /*/// Adds arc to path
    pub fn arc(&mut self, x: f32, y: f32, x1: f32, y1: f32, sweep_angle: f32, x_rotation: f32) {
        let (x, y) = {
            let point = cgmath::point3(x, y, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        let (x1, y1) = {
            let point = cgmath::vec3(x1, y1, 0.);
            let point = self.transform.transform_vector(point);
            (point.x, point.y)
        };
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
    }*/

    /// Adds cubic bezier to path
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = {
            let point = cgmath::point3(x1, y1, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        let (x2, y2) = {
            let point = cgmath::point3(x2, y2, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
        let (x, y) = {
            let point = cgmath::point3(x, y, 0.);
            let point = self.transform.transform_point(point);
            (point.x, point.y)
        };
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

#[derive(Debug)]
struct GeometryStore {
    in_use: HashMap<UniqueGeometry, GeometryBuffer>,
    free: Vec<GeometryBuffer>,
}

impl GeometryStore {
    fn new() -> Self {
        Self {
            in_use: HashMap::new(),
            free: Vec::new(),
        }
    }

    fn malloc(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniq: UniqueGeometry,
        vertices: &[Vertex],
        indices: &[u16],
    ) {
        match self.free.pop() {
            Some(mut buffer) => {
                buffer.write(device, queue, vertices, indices);
                self.in_use.insert(uniq, buffer);
            }
            None => {
                let buffer = GeometryBuffer::new(&device, vertices, indices);
                self.in_use.insert(uniq, buffer);
            }
        }
    }

    fn free(&mut self, uniq: &UniqueGeometry) {
        self.free.push(self.in_use.remove(uniq).unwrap());
    }

    fn free_unused(&mut self, used: &HashSet<&UniqueGeometry>) {
        let mut to_remove = Vec::new();
        for (uniq, _) in self.in_use.iter() {
            if !used.contains(uniq) {
                to_remove.push(uniq.clone());
            }
        }
        for uniq in to_remove {
            self.free(&uniq);
        }
    }
}

// Represents a drawable surface
pub struct Surface {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
}

/// Object that manages the window and GPU resources
pub struct Painter {
    surface: Surface,
    multisampled_framebuffer: wgpu::TextureView,

    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    stack: Vec<Command>,

    geometry_buffers: GeometryStore,

    transform: cgmath::Matrix4<f32>,
    old_transforms: Vec<cgmath::Matrix4<f32>>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    uniform_vec: Vec<Uniforms>,
    uniform_buffer: UniformBuffer,
}

impl Painter {
    pub async fn new_from_window(
        window: &impl raw_window_handle::HasRawWindowHandle,
        size: (u32, u32),
    ) -> Self {
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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.0,
            height: size.1,
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
            geometry_buffers: GeometryStore::new(),
            uniform_bind_group_layout,
            transform: cgmath::Matrix4::identity(),
            old_transforms: Vec::new(),
            multisampled_framebuffer,
            uniform_vec: Vec::new(),
            uniform_buffer: uniform_buffer,
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
        self.old_transforms.push(self.transform);
    }

    pub fn restore(&mut self) {
        self.transform = self.old_transforms.pop().unwrap();
    }

    pub fn reset(&mut self) {
        self.transform = cgmath::Matrix4::identity();
    }

    pub fn fill_text(
        &mut self,
        font: &Font,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        wrap_limit: Option<usize>,
    ) {
        match wrap_limit {
            Some(limit) => assert_ne!(limit, 0),
            None => (),
        }
        let face = font.font.as_face_ref();
        let default_scale = (face.units_per_em().unwrap() as f32).recip();
        let mut path = PathBuilder::new();
        let bbox = face.global_bounding_box();
        let line_height = (bbox.y_min + bbox.y_max) as f32;
        let line_height = line_height + face.capital_height().unwrap() as f32;
        let glyph_width = (bbox.x_min + bbox.x_max) as f32;
        path.translate(x, y);
        path.scale(default_scale * size, default_scale * size);
        path.translate(0., line_height);
        let mut offset = 0.;
        for (i, character) in text.chars().enumerate() {
            if character == '\n' {
                path.translate(-offset, line_height);
                offset = 0.;
                continue;
            }
            let glyph = face.as_face_ref().glyph_index(character).unwrap();
            face.outline_glyph(glyph, &mut path);
            let advance = face.glyph_hor_advance(glyph).unwrap() as f32;
            offset += advance;
            path.translate(advance, 0.);
            match wrap_limit {
                Some(limit) => {
                    if offset > glyph_width * limit as f32 && character == ' ' {
                        path.translate(-offset, line_height);
                        offset = 0.;
                    }
                }
                None => (),
            }
        }
        self.fill_path(&path.build(), color);
    }

    pub fn stroke_text(
        &mut self,
        font: &Font,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        options: StrokeOptions,
        wrap_limit: Option<usize>,
    ) {
        match wrap_limit {
            Some(limit) => assert_ne!(limit, 0),
            None => (),
        }
        let face = font.font.as_face_ref();
        let default_scale = (face.units_per_em().unwrap() as f32).recip();
        let mut path = PathBuilder::new();
        let bbox = face.global_bounding_box();
        let line_height = (bbox.y_min + bbox.y_max) as f32;
        let line_height = line_height + face.capital_height().unwrap() as f32;
        let glyph_width = (bbox.x_min + bbox.x_max) as f32;
        path.translate(x, y);
        path.scale(default_scale * size, default_scale * size);
        path.translate(0., line_height);
        let mut offset = 0.;
        for (i, character) in text.chars().enumerate() {
            if character == '\n' {
                path.translate(-offset, line_height);
                offset = 0.;
                continue;
            }
            let glyph = face.as_face_ref().glyph_index(character).unwrap();
            face.outline_glyph(glyph, &mut path);
            let advance = face.glyph_hor_advance(glyph).unwrap() as f32;
            offset += advance;
            path.translate(advance, 0.);
            match wrap_limit {
                Some(limit) => {
                    if offset > glyph_width * limit as f32 && character == ' ' {
                        path.translate(-offset, line_height);
                        offset = 0.;
                    }
                }
                None => (),
            }
        }
        self.stroke_path(&path.build(), color, options);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.surface.size = new_size;
            self.surface.surface_config.width = new_size.0;
            self.surface.surface_config.height = new_size.1;
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
        use lyon::path::{builder::*, Winding};
        use lyon::tessellation::geometry_builder::BuffersBuilder;
        use lyon::tessellation::*;

        let uniforms =
            Uniforms::from_transform(self.transform, self.surface.size.0, self.surface.size.1);
        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Rectangle(
            OrderedFloat(x),
            OrderedFloat(y),
            OrderedFloat(width),
            OrderedFloat(height),
            color,
        );
        match self.geometry_buffers.in_use.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: buf.indices,
                    path: uniq,
                    vertices: buf.vertices,
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

                builder.add_rectangle(&lyon::math::rect(x, y, width, height), Winding::Positive);

                builder.build().unwrap();

                while geometry.indices.len() * 2 % 4 != 0 {
                    geometry.indices.push(0);
                }

                self.geometry_buffers.malloc(
                    &self.device,
                    &self.queue,
                    uniq.clone(),
                    &geometry.vertices,
                    &geometry.indices,
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                    vertices: geometry.vertices.len(),
                });
            }
        }
    }

    pub fn stroke_path(&mut self, path_builder: &Path, color: Color, options: StrokeOptions) {
        use lyon::tessellation::*;

        let uniforms =
            Uniforms::from_transform(self.transform, self.surface.size.0, self.surface.size.1);

        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq =
            UniqueGeometry::StrokedPath(path_builder.path_instructions.clone(), color, options);
        match self.geometry_buffers.in_use.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    path: uniq,
                    vertices: buf.vertices,
                });
            }
            None => {
                let path = path_builder.path.clone().build();
                let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
                let mut tessellator = StrokeTessellator::new();
                let raw_color = color.as_array();

                {
                    // Compute the tessellation.
                    tessellator
                        .tessellate_path(
                            &path,
                            &options.into(),
                            &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                                Vertex {
                                    position: vertex.position().to_array(),
                                    color: raw_color,
                                }
                            }),
                        )
                        .unwrap();
                }

                while geometry.indices.len() * 2 % 4 != 0 {
                    geometry.indices.push(0);
                }

                self.geometry_buffers.malloc(
                    &self.device,
                    &self.queue,
                    uniq.clone(),
                    &geometry.vertices,
                    &geometry.indices,
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                    vertices: geometry.vertices.len(),
                });
            }
        }
    }

    pub fn fill_path(&mut self, path_builder: &Path, color: Color) {
        use lyon::tessellation::*;

        let uniforms =
            Uniforms::from_transform(self.transform, self.surface.size.0, self.surface.size.1);
        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Path(path_builder.path_instructions.clone(), color);
        match self.geometry_buffers.in_use.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    indices: buf.indices,
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    path: uniq,
                    vertices: buf.vertices,
                });
            }
            None => {
                let path = path_builder.path.clone().build();
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

                while geometry.indices.len() * 2 % 4 != 0 {
                    geometry.indices.push(0);
                }

                self.geometry_buffers.malloc(
                    &self.device,
                    &self.queue,
                    uniq.clone(),
                    &geometry.vertices,
                    &geometry.indices,
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                    vertices: geometry.vertices.len(),
                });
            }
        }
    }

    pub fn circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        use lyon::path::{builder::*, Winding};
        use lyon::tessellation::geometry_builder::BuffersBuilder;
        use lyon::tessellation::*;

        let uniforms =
            Uniforms::from_transform(self.transform, self.surface.size.0, self.surface.size.1);
        self.uniform_vec
            .extend_from_slice(bytemuck::cast_slice(&[uniforms]));

        let uniq = UniqueGeometry::Circle(
            OrderedFloat(x),
            OrderedFloat(y),
            OrderedFloat(radius),
            color,
        );
        match self.geometry_buffers.in_use.get(&uniq) {
            Some(buf) => {
                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: buf.indices,
                    path: uniq,
                    vertices: buf.vertices,
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

                while geometry.indices.len() * 2 % 4 != 0 {
                    geometry.indices.push(0);
                }

                self.geometry_buffers.malloc(
                    &self.device,
                    &self.queue,
                    uniq.clone(),
                    &geometry.vertices,
                    &geometry.indices,
                );

                self.stack.push(Command::RawGeometry {
                    transform: self.transform.clone(),
                    uniform_offset: (self.uniform_vec.len() - 1) as wgpu::BufferAddress,
                    indices: geometry.indices.len(),
                    path: uniq,
                    vertices: geometry.vertices.len(),
                });
            }
        }
    }

    pub fn get_buffer_info(&self) -> String {
        let mut free_backpressure = 0;
        for buf in self.geometry_buffers.free.iter() {
            free_backpressure += buf.vertices + buf.indices;
        }

        let mut used_backpressure = 0;
        for (_, buf) in self.geometry_buffers.in_use.iter() {
            used_backpressure += buf.vertices + buf.indices;
        }

        let total_backpressure = free_backpressure + used_backpressure;
        format!("Buffers free: {}\nBuffers in use: {}\nFree backpressure: {:.5}%\nUsed pressure: {:.5}%", self.geometry_buffers.free.len(), self.geometry_buffers.in_use.len(), (free_backpressure as f32 / total_backpressure as f32) * 100.0, (used_backpressure as f32 / total_backpressure as f32) * 100.0)
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.uniform_vec.clear();
        self.old_transforms.clear();

        self.geometry_buffers = GeometryStore::new();

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

        let mut used = HashSet::new();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.multisampled_framebuffer,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            for (i, command) in self.stack.iter().enumerate() {
                match command {
                    Command::RawGeometry {
                        path,
                        transform: _,
                        indices,
                        uniform_offset,
                        vertices,
                    } => {
                        render_pass.set_bind_group(
                            0,
                            &self.uniform_buffer.bind_group,
                            &[*uniform_offset as u32 * 256 as u32],
                        );

                        let buffers = self.geometry_buffers.in_use.get(&path).unwrap();
                        used.insert(path);
                        render_pass.set_vertex_buffer(
                            0,
                            buffers.vertex_buffer.slice(0..(*vertices) as u64),
                        );
                        render_pass.set_index_buffer(
                            buffers.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(0..*indices as u32, 0, 0..1);
                    }
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        self.uniform_vec.clear();

        self.old_transforms.clear();
        self.geometry_buffers.free_unused(&used);
        self.reset();
        self.stack.clear();
        //console_output(&self.latest_profiler_results);

        Ok(())
    }
}

fn scopes_to_console_recursive(results: &[GpuTimerScopeResult], indentation: u32) {
    for scope in results {
        if indentation > 0 {
            print!("{:<width$}", "|", width = 4);
        }
        println!(
            "{:.3}μs - {}",
            (scope.time.end - scope.time.start) * 1000.0 * 1000.0,
            scope.label
        );
        if !scope.nested_scopes.is_empty() {
            scopes_to_console_recursive(&scope.nested_scopes, indentation + 1);
        }
    }
}

fn console_output(results: &Option<Vec<GpuTimerScopeResult>>) {
    print!("\x1B[2J\x1B[1;1H"); // Clear terminal and put cursor to first row first column
    println!("Welcome to wgpu_profiler demo!");
    println!(
        "Press space to write out a trace file that can be viewed in chrome's chrome://tracing"
    );
    println!();
    match results {
        Some(results) => {
            scopes_to_console_recursive(&results, 0);
        }
        None => println!("No profiling results available yet!"),
    }
}
