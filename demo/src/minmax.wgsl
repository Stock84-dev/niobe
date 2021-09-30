[[block]]
struct Args {
    min_offset: u32;
    max_offset: u32;
    stride: u32;
    range_end_excluding: u32;
    dest_offset: u32;
};
struct Point {
    [[size(4)]] val: f32;
};
[[block]]
struct Data {
    data: array<f32>;
};

// [[group(0), binding(1)]] var<storage, read_write> dest : Output;
[[group(0), binding(0)]] var<uniform> args : Args;
[[group(0), binding(1)]] var<storage, read> src : Data;
[[group(0), binding(2)]] var<storage, read_write> dest : Data; // TODO: how to set this bind to be write only on host side ?
var<workgroup> sdata: array<f32, 128>;

// limits: max number of elements = (2^32-1 - max(min_offset, max_offset)) / stride
// we set work group size to 64 to support most devices
// number of work groups needs to be adjusted for each device
// pipeline example:
// maximum sized
// call it with max number of work groups 2^16, this produces 2^16 items, requires at least 2^16*64 items
// dynamically size number of work groups such that each work group processes one batch
// then call it again with 2^16/64 = 1024 work groups, this produces 16 items
// then call it again with one work group, this produces one item
// 2^16 - max allowed work group size by wgpu
// stage 1: < 2^16 work groups -> < 2^16 results, n_elements >= 2^16*64
// stage 2: < 64 work groups -> < 64 results, n_elements < 2^16*64
// stage 3: one work group -> one result, n_elements >= 64
// this isn't the most efficient way but it gets the job done in regards to number of batches per work group
// but there isn't a way to get number of SMs/CUDA cores for a device to calculate correctly
// TODO: we could also specialize(using templates at compile time) shaders to remove while loop when, stride and offset, also specialize for 2D data
// code modified from https://developer.download.nvidia.com/assets/cuda/files/reduction.pdf
// and from https://github.com/sschaetz/nvidia-opencl-examples/blob/master/OpenCL/src/oclReduction/oclReduction_kernel.cl
[[stage(compute), workgroup_size(16)]]
fn minmax64(
    [[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>,
    [[builtin(local_invocation_id)]] local_invocation_id: vec3<u32>,
    [[builtin(workgroup_id)]] workgroup_id: vec3<u32>,
    [[builtin(num_workgroups)]] num_workgroups: vec3<u32>,
) {
    let gid = global_invocation_id.x;
    let lid = local_invocation_id.x;
    let maxid = lid + 64u;
    let wgid = workgroup_id.x;
    let nwg = num_workgroups.x;
    // across whole device each thread is sequentially accesing
    var min_i: u32 = gid * args.stride + args.min_offset;
    var max_i: u32 = gid * args.stride + args.max_offset;
    // let lws = 2;
    // let gws = 30;
    // let nwg = 15;
    // element
    // gid  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29
    // lid  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1
    // wgid 0  0  1  1  2  2  3  3  4  4  5  5  6  6  7  7  8  8  9  9 10 10 11 11 12 12 13 13 14 14
    // i is positioned relative to wgid then gets increased by grid size
    // block size = lws
    let stride = (64u * nwg) * args.stride;
    sdata[lid] = src.data[min_i];
    sdata[maxid] = src.data[max_i];
    min_i = min_i + stride;
    max_i = max_i + stride;
    // loop {
    //     if (max_i > args.range_end_excluding) {         
    //         break;
    //     }
    //     sdata[lid] = max(sdata[lid], src.data[min_i]);
    //     sdata[maxid] = max(sdata[maxid], src.data[max_i]);
    //     min_i = min_i + stride;
    //     max_i = max_i + stride;
    // } 
    workgroupBarrier();
    if (lid < 8u) {
        // sdata[lid] = max(sdata[lid], sdata[lid + 32u]);
        // sdata[lid] = sdata[lid + 32u];
        sdata[lid] = sdata[lid + 8u];
    } 
    workgroupBarrier();
    // if (lid >= 32u) {
    //     sdata[lid] = 0.;
    // }
    // if (lid < 32u) { 
    //     sdata[lid] = max(sdata[lid], sdata[lid + 32u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid + 32u]); 
    // } 
    // workgroupBarrier();
    // if (lid < 16u) {
    //     sdata[lid] = max(sdata[lid], sdata[lid + 16u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid + 16u]); 
    // }
    // workgroupBarrier();
    // if (lid <  8u) {
    //     sdata[lid] = max(sdata[lid], sdata[lid +  8u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid +  8u]); 
    // }
    // workgroupBarrier();
    // if (lid <  4u) {
    //     sdata[lid] = max(sdata[lid], sdata[lid +  4u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid +  4u]); 
    // }
    // workgroupBarrier();
    // if (lid <  2u) {
    //     sdata[lid] = max(sdata[lid], sdata[lid +  2u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid +  2u]); 
    // }
    // workgroupBarrier();
    // if (lid <  1u) {
    //     sdata[lid] = max(sdata[lid], sdata[lid +  1u]); 
    //     // sdata[maxid] = max(sdata[maxid], sdata[maxid +  1u]); 
    //     dest.data[wgid * 2u + args.dest_offset] = sdata[lid];
    //     // dest.data[wgid * 2u + args.dest_offset + 1u] = sdata[maxid];
    // }
    workgroupBarrier();
    dest.data[lid] = sdata[lid];
    // let stride = (64u * nwg) * args.stride;
    // sdata[lid] = src.data[min_i];
    // sdata[maxid] = src.data[max_i];
    // min_i = min_i + stride;
    // max_i = max_i + stride;
    // loop {
    //     if (max_i > args.range_end_excluding) {         
    //         break;
    //     }
    //     sdata[lid] = min(sdata[lid], src.data[min_i]);
    //     sdata[maxid] = max(sdata[maxid], src.data[max_i]);
    //     min_i = min_i + stride;
    //     max_i = max_i + stride;
    // } 
    // workgroupBarrier();
    // if (lid < 32u) { 
    //     sdata[lid] = min(sdata[lid], sdata[lid + 32u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid + 32u]); 
    // } 
    // workgroupBarrier();
    // if (lid < 16u) {
    //     sdata[lid] = min(sdata[lid], sdata[lid + 16u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid + 16u]); 
    // }
    // workgroupBarrier();
    // if (lid <  8u) {
    //     sdata[lid] = min(sdata[lid], sdata[lid +  8u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid +  8u]); 
    // }
    // workgroupBarrier();
    // if (lid <  4u) {
    //     sdata[lid] = min(sdata[lid], sdata[lid +  4u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid +  4u]); 
    // }
    // workgroupBarrier();
    // if (lid <  2u) {
    //     sdata[lid] = min(sdata[lid], sdata[lid +  2u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid +  2u]); 
    // }
    // workgroupBarrier();
    // if (lid <  1u) {
    //     sdata[lid] = min(sdata[lid], sdata[lid +  1u]); 
    //     sdata[maxid] = max(sdata[maxid], sdata[maxid +  1u]); 
    //     dest.data[wgid * 2u + args.dest_offset] = sdata[lid];
    //     dest.data[wgid * 2u + args.dest_offset + 1u] = sdata[maxid];
    // }
}