# bufro
[![crates.io](https://img.shields.io/crates/v/bufro.svg)](https://crates.io/crates/bufro)
[![Documentation](https://docs.rs/bufro/badge.svg)](https://docs.rs/bufro)


A vector graphics renderer using wgpu with a Rust &amp; C API.

A Rust example can be found in examples/quickstart.rs (using winit).
A C example can be found in c\_examples/quickstart.c (using glfw). Build the C examples by running `make <example>` in the c_examples folder.

## Roadmap

- [x] Transformations (e.g. ctx.rotate)
- [x] Rectangle fill
- [x] Circle fill
- [x] Blending
- [X] Strokes
- [X] Text rendering (stroke & fill)
- [ ] Effects (glow & shadows)
- [ ] Gradients
- [X] Rounded rectangles and polygons
- [ ] Use Lyon for tesselation
- [ ] Custom shader language
- [X] Web API (using wasm-bindgen)


## Demo

![Gif showing bufro in action](https://raw.githubusercontent.com/UE2020/bufro/main/demo.gif)

## Build &amp; Install (C)

```sh
$ make # Build the dynamic library and generate the C header
% make install # Install the header and library system-wide 
```
*Note that $ indicates a regular user shell, while % denotes a root shell.*
