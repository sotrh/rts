// Ignored: only used between vertex and fragment shader 
// - Could be necessary if using shaders from different files
struct VertexOutput {
    [[location(0)]]
    uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec3<f32>,
    [[location(1)]] uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = uv;
    out.position = locals.transform * vec4<f32>(pos, 1.0);
    return out;
}

// Can group/bindings conflict in the same file?
[[group(0), binding(1)]]
var color: texture_2d<f32>;
[[group(0), binding(2)]]
var sampler: sampler;

// How to get dependencies of an entry point?
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(color, sampler, in.uv);
}