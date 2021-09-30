
[[block]]
struct Uniform {
    color: vec4<f32>;
    scale: vec2<f32>;
    translate: vec2<f32>;
    line_scale: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uni: Uniform;

struct VertexInput {
    [[location(0)]] pos: vec2<f32>;
};

struct InstanceInput {
    [[location(1)]] first: vec2<f32>;
    [[location(2)]] second: vec2<f32>;
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
    // see: https://wwwtyro.net/2019/11/18/instanced-lines.html
    // length between 2 points
    let xBasis = instance.second - instance.first;
    // normal is a vector that is perpendicular over another vector
    var yBasis: vec2<f32> = normalize(vec2<f32>(-xBasis.y, xBasis.x)); // direction of a normal
    // TODO: prebake width into quad by multiplying all y values with width to avoid multiplying with it in shader
    // How to render a line with borders: render thicker line, then render thinner line with the same data
    // How to render a line with custom geometry at joints: render line then using different shader we draw with custom instance over same point data using only one vao for a point to not draw 2 times at the same spot
    let pos = instance.first + xBasis * model.pos.x + yBasis * uni.line_scale * model.pos.y;
    out.clip_position = vec4<f32>(pos * uni.scale + uni.translate, 1.0, 1.0);
    return out;
}

// Fragment shader
[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return uni.color;
}