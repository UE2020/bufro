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

![Image of demo](https://raw.githubusercontent.com/UE2020/bufro/main/demo.png)

## Build &amp; Install (C)

```sh
$ make # Build the dynamic library and generate the C header
% make install # Install the header and library system-wide 
```
*Note that $ indicates a regular user shell, while % denotes a root shell.*
