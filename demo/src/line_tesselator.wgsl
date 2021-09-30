[[block]]
struct Data {
    points : [[stride(4)]] array<f32>;
};

// [[group(0), binding(1)]] var<storage, read_write> dest : Output;
[[group(0), binding(0)]] var<storage, read> points : Data;
[[group(0), binding(1)]] var<storage, read_write> dest : Data;

[[stage(compute), workgroup_size(64)]]
fn main64([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let gid = global_invocation_id.x;
  let lid = local_invocation_id.x;
  let wgid = workgroup_id.x;
  // --- elements
  // 64 left 64 right
  let left_min = points.points[wgid * 64 * 4 + lid];
  let left_max = points.points[wgid * 64 * 4 + 64 + lid];
  let right_min = points.points[wgid * 64 * 4 + 64 * 2 + lid];
  let right_max = points.points[wgid * 64 * 4 + 64 * 3 + lid];

  dest.points[wgid * 64 + lid] = min(left_min, right_min);
  dest.points[wgid * 64 + 64 + lid] = max(left_max, right_max);
}

[[stage(compute), workgroup_size(1)]]
fn main1() {
    var final_min: f32 = points.points[0];
    var final_max: f32 = points.points[1];
    for (var i : u8 = 1u; i < 64; i = i + 1u) {
        let right_min = points.points[i * 2];
        let right_max = points.points[i * 2 + 1];
        final_min = min(final_min, right_min);
        final_max = max(final_max, right_max);
    }
  dest[0] = final_min;
  dest[1] = final_max;
}