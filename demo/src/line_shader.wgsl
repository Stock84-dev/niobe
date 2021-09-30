
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
    // scaling the length by vertex x value
    // x is either 0 or 1
    // scaling normal by width gives us our desired line width
    let pos = instance.first + xBasis * model.pos.x + yBasis * uni.line_scale * model.pos.y;
    // let pos = instance.first + xBasis * model.pos.x * uni.scale + yBasis * uni.line_width * model.pos.y * uni.scale;
    // let pos = instance.first + xBasis * model.pos.x + yBasis * uni.line_width * model.pos.y;
    // let pos = model.pos / 100. + instance.second;

    // let normal = vec2<f32>(instance.first.y - instance.second.y, instance.second.x - instance.first.x);
    // var pos: vec2<f32> = (model.pos + normal) / 10.;
    // // [Ay - By, Bx - Ax]
    // // pos = pos + instance.second;
    // if (vid == 0u) {
    //     pos = pos + instance.first;
    // }
    // if (vid == 1u) {
    //     pos = pos + instance.first;
    // }
    // if (vid == 2u) {
    //     pos = pos + instance.second + instance.first;
    // }
    // if (vid == 3u) {
    //     pos = pos + instance.second + instance.first;
    // }

    out.clip_position = vec4<f32>(pos * uni.scale + uni.translate, 1.0, 1.0);
    // out.clip_position = vec4<f32>(pos + uni.translate, 1.0, 1.0);
    // out.clip_position = vec4<f32>(point, 1.0, 1.0);
    // out.color = vec4<f32>(model.color, 1.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // var out: vec4<f32> = vec4<f32>(1., 0., 0., 0.5);
    // return out;
    // return in.color;
    return uni.color;
    // return vec4<f32>(1., 0., 0., 1.0);
}



//// vertex storage buffers aren't supported on all devices
//// [[group(0), binding(1)]] var<storage, read_write> data : Data;
//// Vertex shader
//
//// data must go in vertex buffer
//// we can store vertex offsets in uniform then use
//
//// 2 vertex buffers, one holds current data, other one holds next data
//
//// uniform buffers are read only from shader
//[[block]]
//struct Uniform {
//    scale: vec2<f32>;
//    translate: vec2<f32>;
//    line_width: f32;
//};
//
//[[group(0), binding(0)]]
//var<uniform> uni: Uniform;
//
//struct VertexInput {
//    [[location(0)]] pos: vec2<f32>;
//    [[location(1)]] normal: vec2<f32>;
//    [[location(2)]] color: vec3<f32>;
//};
//
//struct VertexOutput {
//    [[builtin(position)]] clip_position: vec4<f32>;
//    [[location(0)]] color: vec4<f32>;
//};
//
//[[stage(vertex)]]
//fn main(
//    [[builtin(vertex_index)]] vid: u32,
//    model: VertexInput,
//) -> VertexOutput {
//    var out: VertexOutput;
//    let point = model.pos;
//
//    out.clip_position = vec4<f32>((point + model.normal) * uni.scale + uni.translate, 1.0, 1.0);
//    // out.clip_position = vec4<f32>(point, 1.0, 1.0);
//    out.color = vec4<f32>(model.color, 1.0);
//    return out;
//}
//
//// Fragment shader
//
//[[stage(fragment)]]
//fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
//    // var out: vec4<f32> = vec4<f32>(1., 0., 0., 0.5);
//    // return out;
//    return in.color;
//    // return vec4<f32>(1., 0., 0., 1.0);
//}

// template <class T>
// T profileReduce(ReduceType datatype,
//                   cl_int  n, // number of elements
//                   int  numThreads, 
//                   int  numBlocks,
//                   int  maxThreads,
//                   int  maxBlocks,
//                   int  whichKernel, 
//                   int  testIterations,
//                   bool cpuFinalReduction,
//                   int  cpuFinalThreshold,
//                   double* dTotalTime,
//                   T* h_odata,
//                   cl_mem d_idata, 
//                   cl_mem d_odata)
// {

//  threads = (n < maxThreads*2) ? nextPow2((n + 1)/ 2) : maxThreads;
// blocks = MIN(maxBlocks, blocks)
// at the start blockSize = numThreads
// local buffer is numThreads in len which is block size

// globalWorkSize[0] = numBlocks * numThreads;
//localWorkSize[0] = numThreads;
// numThreads = lws
// numBlocks = n_work_groups

// blockSize - number of elements per thread
// __kernel void reduce6(__global T *g_idata, __global T *g_odata, unsigned int n, __local volatile T* sdata)
// {
//     // perform first level of reduction,
//     // reading from global memory, writing to shared memory
//     unsigned int tid = get_local_id(0);
//     unsigned int i = get_group_id(0)*(get_local_size(0)*2) + get_local_id(0);
//     unsigned int gridSize = blockSize*2*get_num_groups(0);
//     sdata[tid] = 0;

//     // we reduce multiple elements per thread.  The number is determined by the 
//     // number of active thread blocks (via gridDim).  More blocks will result
//     // in a larger gridSize and therefore fewer elements per thread
//     while (i < n)
//     {         
//         sdata[tid] += g_idata[i];
//         // ensure we don't read out of bounds -- this is optimized away for powerOf2 sized arrays
//         if (nIsPow2 || i + blockSize < n) 
//             sdata[tid] += g_idata[i+blockSize];  
//         i += gridSize;
//     } 

//     barrier(CLK_LOCAL_MEM_FENCE);

//     // do reduction in shared mem
//     if (blockSize >= 512) { if (tid < 256) { sdata[tid] += sdata[tid + 256]; } barrier(CLK_LOCAL_MEM_FENCE); }
//     if (blockSize >= 256) { if (tid < 128) { sdata[tid] += sdata[tid + 128]; } barrier(CLK_LOCAL_MEM_FENCE); }
//     if (blockSize >= 128) { if (tid <  64) { sdata[tid] += sdata[tid +  64]; } barrier(CLK_LOCAL_MEM_FENCE); }
    
//     if (tid < 32)
//     {
//         if (blockSize >=  64) { sdata[tid] += sdata[tid + 32]; }
//         if (blockSize >=  32) { sdata[tid] += sdata[tid + 16]; }
//         if (blockSize >=  16) { sdata[tid] += sdata[tid +  8]; }
//         if (blockSize >=   8) { sdata[tid] += sdata[tid +  4]; }
//         if (blockSize >=   4) { sdata[tid] += sdata[tid +  2]; }
//         if (blockSize >=   2) { sdata[tid] += sdata[tid +  1]; }
//     }
    
//     // write result for this block to global mem 
//     if (tid == 0) g_odata[get_group_id(0)] = sdata[0];
// }
