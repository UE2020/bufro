# bufro

A vector graphics renderer using OpenGL with a Rust &amp; C API.

A Rust example can be found in examples/quickstart.rs (using glutin).
A C example can be found in c\_examples/quickstart/quickstart.c (using glfw).

## Roadmap

Mostly unfinished.

- [x] Transformations (e.g. ctx.rotate)
- [x] Rectangle fill
- [x] Circle fill
- [ ] Circle stroke
- [ ] Rectangle stroke
- [ ] Effects (glow & shadows)
- [ ] Gradients
- [ ] Rounded rectangle
- [ ] Use Lyon for tesselation
- [ ] Custom shader language
- [ ] Web API (using wasm-bindgen)


## Demo

[![Demo](https://res.cloudinary.com/marcomontalbano/image/upload/v1620821432/video_to_markdown/images/video--2fb2cf0e3ecdf60d7038944e1a9f85e3-c05b58ac6eb4c4700831b2b3070cd403.jpg)](https://raw.githubusercontent.com/UE2020/bufro/main/demo.mp4 "Demo")

## Build &amp; Install (C)

```sh
$ make # Build the dynamic library and generate the C header
% make install # Install the header and library system-wide 
```
*Note that $ indicates a regular user shell, while % denotes a root shell.*
