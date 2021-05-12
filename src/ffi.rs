use std::ffi::c_void;

use crate::*;

#[no_mangle]
pub unsafe extern "C" fn bfr_create_surface(
    loader: extern "C" fn(*const libc::c_char) -> *const c_void,
) -> *mut Renderer {
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
    (*renderer).gl.delete_vertex_array((*renderer).vertex_array); // TODO: Fix
    (*renderer).gl.delete_buffer((*renderer).vertex_buffer); // TODO: Fix

    // free renderer
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

#[no_mangle]
pub unsafe extern "C" fn bfr_color8(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_8(r, g, b, a)
}

#[no_mangle]
pub unsafe extern "C" fn bfr_colorf(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::from_f(r, g, b, a)
}

#[no_mangle]
pub unsafe extern "C" fn bfr_set_clear_color(renderer: *mut Renderer, color: Color) {
    (*renderer).set_clear_color(color);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_rect(
    renderer: *mut Renderer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    angle: f32,
    color: Color,
) {
    (*renderer).rect(x, y, width, height, angle, color);
}

// transforms

#[no_mangle]
pub unsafe extern "C" fn bfr_translate(renderer: *mut Renderer, x: f32, y: f32) {
    (*renderer).translate(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_scale(renderer: *mut Renderer, x: f32, y: f32) {
    (*renderer).scale(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_rotate(renderer: *mut Renderer, x: f32) {
    (*renderer).rotate(x);
}

#[no_mangle]
pub unsafe extern "C" fn bfr_save(renderer: *mut Renderer) {
    (*renderer).save();
}


#[no_mangle]
pub unsafe extern "C" fn bfr_restore(renderer: *mut Renderer) {
    (*renderer).restore();
}


#[no_mangle]
pub unsafe extern "C" fn bfr_reset(renderer: *mut Renderer) {
    (*renderer).reset();
}
