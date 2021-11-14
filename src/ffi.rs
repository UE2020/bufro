use std::{ffi::c_void, mem::MaybeUninit};

use crate::*;

use std::ffi::{CStr, CString};

use libc::{c_ulong, c_char};

pub struct BufroFont(Font);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct BufroXlibWindow {
    pub window: c_ulong,
    pub display: *mut c_void,
}

impl Into<raw_window_handle::RawWindowHandle> for BufroXlibWindow {
    fn into(self) -> raw_window_handle::RawWindowHandle {
        raw_window_handle::RawWindowHandle::Xlib(raw_window_handle::unix::XlibHandle {
            window: self.window,
            display: self.display,
            ..raw_window_handle::unix::XlibHandle::empty()
        })
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for BufroXlibWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        (*self).into()
    }
}

#[no_mangle]
pub unsafe extern "C" fn bfr_painter_from_xlib_window(
    handle: BufroXlibWindow,
    width: u32,
    height: u32,
) -> *mut Painter {
    let painter = Box::new(pollster::block_on(Painter::new_from_window(&handle, (width, height))));
    Box::into_raw(painter)
}

/// free painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_free(painter: *mut Painter) {
    Box::from_raw(painter);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_font_from_buffer(
    data: *const c_char,
    len: usize,
    ptr: *mut *mut BufroFont,
) -> u8 {
    let data = data as *const u8;
    let data = &*std::ptr::slice_from_raw_parts(data, len);
    let boxed = Box::new({
        match Font::new(data) {
            Some(font) => BufroFont(font),
            None => return 1
        }
    });
    *ptr = Box::into_raw(boxed);
    0
}

#[no_mangle]
pub unsafe extern "C" fn bfr_painter_resize(
    painter: *mut Painter,
    width: u32,
    height: u32,
) {
    (*painter).resize((width, height));
}

#[no_mangle]
pub unsafe extern "C" fn bfr_painter_rectangle(
    painter: *mut Painter,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: BufroColor,
) {
    (*painter).rectangle(x, y, width, height, std::mem::transmute(color));
}

#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_new() -> *mut PathBuilder {
    Box::into_raw(Box::new(PathBuilder::new()))
}

/// free pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_free(pathbuilder: *mut PathBuilder) {
    Box::from_raw(pathbuilder);
}

/// TODO: This is a hack.
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_build(
    pathbuilder: *mut PathBuilder,
    path: *mut *mut Path,
) {
    let mut builder: MaybeUninit<PathBuilder> = MaybeUninit::uninit().assume_init();
    std::ptr::copy(pathbuilder, &mut builder as *mut MaybeUninit<PathBuilder> as *mut PathBuilder, 1);
    let builder = builder.assume_init();
    let newpath = Box::new(builder.build());
    *path = Box::into_raw(newpath);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_scale(pathbuilder: *mut PathBuilder, x: f32, y: f32) {
    (*pathbuilder).scale(x, y);
}

/// rotate pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_rotate(pathbuilder: *mut PathBuilder, angle: f32) {
    (*pathbuilder).rotate(angle);
}

/// translate pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_translate(pathbuilder: *mut PathBuilder, x: f32, y: f32) {
    (*pathbuilder).translate(x, y);
}

/// save pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_save(pathbuilder: *mut PathBuilder) {
    (*pathbuilder).save();
}

/// restore pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_restore(pathbuilder: *mut PathBuilder) {
    (*pathbuilder).restore();
}

/// close pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_close(pathbuilder: *mut PathBuilder) {
    (*pathbuilder).close();
}

/// moveto pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_move_to(pathbuilder: *mut PathBuilder, x: f32, y: f32) {
    (*pathbuilder).move_to(x, y);
}

/// lineto pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_line_to(pathbuilder: *mut PathBuilder, x: f32, y: f32) {
    (*pathbuilder).line_to(x, y);
}

/// quadto pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_quad_to(
    pathbuilder: *mut PathBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) {
    (*pathbuilder).quad_to(x1, y1, x2, y2);
}

/// curveto pathbuilder
#[no_mangle]
pub unsafe extern "C" fn bfr_pathbuilder_curve_to(
    pathbuilder: *mut PathBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) {
    (*pathbuilder).curve_to(x1, y1, x2, y2, x3, y3);
}

/// scale painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_scale(painter: *mut Painter, x: f32, y: f32) {
    (*painter).scale(x, y);
}

/// rotate painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_rotate(painter: *mut Painter, angle: f32) {
    (*painter).rotate(angle);
}

/// translate painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_translate(painter: *mut Painter, x: f32, y: f32) {
    (*painter).translate(x, y);
}

/// save painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_save(painter: *mut Painter) {
    (*painter).save();
}

/// restore painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_restore(painter: *mut Painter) {
    (*painter).restore();
}

/// fill text on painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_fill_text(
    painter: *mut Painter,
    font: *const BufroFont,
    text: *const c_char,
    x: f32,
    y: f32,
    size: f32,
    color: BufroColor,
    wrap_limit: usize,
) {

    let text = CStr::from_ptr(text);
    let text = text.to_str().unwrap();
    (*painter).fill_text(&(*font).0, text, x, y, size, std::mem::transmute(color), match wrap_limit {
        0 => None,
        _ => Some(wrap_limit),
    });
}

/// stroke text on painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_stroke_text(
    painter: *mut Painter,
    font: *const BufroFont,
    text: *const c_char,
    x: f32,
    y: f32,
    size: f32,
    color: BufroColor,
    options: BufroStrokeOptions,
    wrap_limit: usize,
) {
    let text = CStr::from_ptr(text);
    let text = text.to_str().unwrap();
    (*painter).stroke_text(&(*font).0, text, x, y, size, std::mem::transmute(color), std::mem::transmute(options), match wrap_limit {
        0 => None,
        _ => Some(wrap_limit),
    });
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
#[repr(C)]
pub enum BufroLineCap {
    /// The stroke for each sub-path does not extend beyond its two endpoints.
    /// A zero length sub-path will therefore not have any stroke.
    BufroLineCapButt,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a rectangle with the same width as the stroke width and
    /// whose length is half of the stroke width. If a sub-path has zero length,
    /// then the resulting effect is that the stroke for that sub-path consists
    /// solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    BufroLineCapSquare,
    /// At each end of each sub-path, the shape representing the stroke will be extended
    /// by a half circle with a radius equal to the stroke width.
    /// If a sub-path has zero length, then the resulting effect is that the stroke for
    /// that sub-path consists solely of a full circle centered at the sub-path's point.
    BufroLineCapRound,
}

/// Line join as defined by the SVG specification.
///
/// See: <https://svgwg.org/specs/strokes/#StrokeLinejoinProperty>
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
#[repr(C)]
pub enum BufroLineJoin {
    /// A sharp corner is to be used to join path segments.
    BufroLineJoinMiter,
    /// Same as a miter join, but if the miter limit is exceeded,
    /// the miter is clipped at a miter length equal to the miter limit value
    /// multiplied by the stroke width.
    BufroLineJoinMiterClip,
    /// A round corner is to be used to join path segments.
    BufroLineJoinRound,
    /// A bevelled corner is to be used to join path segments.
    /// The bevel shape is a triangle that fills the area between the two stroked
    /// segments.
    BufroLineJoinBevel,
}

#[repr(C)]
pub struct BufroStrokeOptions {
    /// What cap to use at the start of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub start_cap: BufroLineCap,

    /// What cap to use at the end of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub end_cap: BufroLineCap,

    /// See the SVG specification.
    ///
    /// Default value: `LineJoin::Miter`.
    pub line_join: BufroLineJoin,

    /// Line width
    ///
    /// Default value: `StrokeOptions::DEFAULT_LINE_WIDTH`.
    pub line_width: f32,

    /// See the SVG specification.
    ///
    /// Must be greater than or equal to 1.0.
    /// Default value: `StrokeOptions::DEFAULT_MITER_LIMIT`.
    pub miter_limit: f32,
}

/// regen painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_regen(painter: *mut Painter) {
    (*painter).regen();
}

/// stroke path on painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_stroke_path(
    painter: *mut Painter,
    path: *const Path,
    color: BufroColor,
    options: BufroStrokeOptions,
) {
    (*painter).stroke_path(&*path, std::mem::transmute(color), std::mem::transmute(options));
}

/// fill path on painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_fill_path(painter: *mut Painter, path: *const Path, color: BufroColor) {
    (*painter).fill_path(&*path, std::mem::transmute(color));
}

/// circle on painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_circle(
    painter: *mut Painter,
    x: f32,
    y: f32,
    radius: f32,
    color: BufroColor,
) {
    (*painter).circle(x, y, radius, std::mem::transmute(color),);
}

/// get buffer info string from painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_get_buffer_info_string(painter: *mut Painter) -> *const c_char {
    let info = (*painter).get_buffer_info();
    CString::new(info).unwrap().into_raw()
}

/// clear painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_clear(painter: *mut Painter) {
    (*painter).clear();
}

/// flush painter
#[no_mangle]
pub unsafe extern "C" fn bfr_painter_flush(painter: *mut Painter) -> BufroFlushResult {
    match (*painter).flush() {
        Ok(()) => BufroFlushResult::BufroFlushResultOk,
        Err(e) => BufroFlushResult::from(e),
    }
}

#[repr(C)]
pub enum BufroFlushResult {
    BufroFlushResultTimeout,
    BufroFlushResultOutdated,
    BufroFlushResultLost,
    BufroFlushResultOutOfMemory,
    BufroFlushResultOk,
}

impl From<wgpu::SurfaceError> for BufroFlushResult {
    fn from(err: wgpu::SurfaceError) -> Self {
        match err {
            wgpu::SurfaceError::Outdated => BufroFlushResult::BufroFlushResultOutdated,
            wgpu::SurfaceError::Lost => BufroFlushResult::BufroFlushResultLost,
            wgpu::SurfaceError::OutOfMemory => BufroFlushResult::BufroFlushResultOutOfMemory,
            wgpu::SurfaceError::Timeout => BufroFlushResult::BufroFlushResultTimeout,
        }
    }
}

#[repr(C)]
pub struct BufroColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// bufro color from floats
#[no_mangle]
pub unsafe extern "C" fn bfr_colorf(r: f32, g: f32, b: f32, a: f32) -> BufroColor {
    BufroColor { r, g, b, a }
}

/// bufro color from u8s
#[no_mangle]
pub unsafe extern "C" fn bfr_coloru8(r: u8, g: u8, b: u8, a: u8) -> BufroColor {
    BufroColor {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}