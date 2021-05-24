# bufro
[![crates.io](https://img.shields.io/crates/v/bufro.svg)](https://crates.io/crates/bufro)
[![Documentation](https://docs.rs/bufro/badge.svg)](https://docs.rs/bufro)


A vector graphics renderer using OpenGL with a Rust &amp; C API.

A Rust example can be found in examples/quickstart.rs (using glutin).
A C example can be found in c\_examples/quickstart.c (using glfw). Build the C examples by running `make <example>` in the c_examples folder.

## Roadmap

Mostly unfinished.

- [x] Transformations (e.g. ctx.rotate)
- [x] Rectangle fill
- [x] Circle fill
- [x] Blending
- [ ] Strokes
- [ ] Effects (glow & shadows)
- [ ] Gradients
- [ ] Rounded rectangles and polygons
- [ ] Use Lyon for tesselation
- [ ] Custom shader language
- [ ] Web API (using wasm-bindgen)


## Demo

![Gif showing bufro in action](https://raw.githubusercontent.com/UE2020/bufro/main/demo.gif)

## Build &amp; Install (C)

```sh
$ make # Build the dynamic library and generate the C header
% make install # Install the header and library system-wide 
```
*Note that $ indicates a regular user shell, while % denotes a root shell.*
