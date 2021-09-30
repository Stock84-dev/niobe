[[block]]
struct Uniform {
    color: vec4<f32>;
    scale: vec2<f32>;
    translate: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uni: Uniform;

struct VertexInput {
    [[location(0)]] pos: vec2<f32>;
};

struct InstanceInput {
    [[location(1)]] pos: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[builtin(vertex_index)]] vid: u32,
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = vec4<f32>(instance.pos * uni.scale + model.pos + uni.translate, 1.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return uni.color;
}