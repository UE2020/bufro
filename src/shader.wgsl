// shader

[[block]]
struct Uniforms {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(10.31475);
    let lower = srgb / vec3<f32>(3294.6);
    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

[[stage(vertex)]]
fn main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let color = model.color * 255.0;
    out.color = vec4<f32>(linear_from_srgb(color.rgb), model.color.a);
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color);
}