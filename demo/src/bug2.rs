use std::sync::Arc;
use std::thread;
use wgpu::{BufferAddress, Maintain, MapMode};
macro_rules! src {
    () => {
        "\
[[block]]
struct Data {{
    data: array<u32>;
}};
[[group(0), binding(0)]] var<storage, read_write> dest : Data;

[[stage(compute), workgroup_size({size})]]
fn main(
    [[builtin(local_invocation_id)]] local_invocation_id: vec3<u32>,
) {{
    let lid = local_invocation_id.x;
    dest.data[lid] = lid;
    workgroupBarrier();
    if (lid < {half}u) {{
        dest.data[lid] = dest.data[lid + {half}u];
        // dest.data[lid] = dest.data[lid] * 2u; // this line would work fine
    }}
}}
 "
    };
}

pub fn main() -> anyhow::Result<()> {
    pollster::block_on(bug())?;
    Ok(())
}

pub async fn bug() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();
    let good_size = 8;
    let bad_size = 128;
    let good_src = format!(src!(), size = good_size, half = good_size / 2);
    let bad_src = format!(src!(), size = bad_size / 8, half = bad_size / 16);
    println!("{}", good_src);
    println!("{}", bad_src);

    println!("{:#?}", adapter.get_info());
    let device = Arc::new(device);
    let dest_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: (std::mem::size_of::<f32>() * bad_size) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let d = device.clone();
    thread::spawn(move || loop {
        d.poll(Maintain::Wait);
    });

    let good_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(good_src.into()),
    });
    let bad_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(bad_src.into()),
    });

    let compute_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("compute"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });

    let good_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &good_shader,
        entry_point: "main",
    });
    let bad_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &bad_shader,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &compute_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: dest_buffer.as_entire_binding(),
        }],
        label: None,
    });
    //
    //    let mut command_encoder =
    //        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    //    let mut cpass =
    //        command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
    //    cpass.set_pipeline(&good_pipeline);
    //    cpass.set_bind_group(0, &bind_group, &[]);
    //    cpass.dispatch(1 as _, 1, 1);
    //    drop(cpass);
    //    queue.submit(Some(command_encoder.finish()));
    //    let slice = dest_buffer.slice(..(good_size / 2 * std::mem::size_of::<u32>()) as BufferAddress);
    //    slice.map_async(MapMode::Read).await?;
    //    let map = slice.get_mapped_range();
    //    let good_output: Vec<u32> = bytemuck::cast_slice(map.as_ref()).to_vec();
    //    drop(map);
    //    dest_buffer.unmap();

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let mut cpass =
        command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
    cpass.set_pipeline(&bad_pipeline);
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.dispatch(1 as _, 1, 1);
    drop(cpass);
    queue.submit(Some(command_encoder.finish()));
    let slice = dest_buffer.slice(..(bad_size / 2 * std::mem::size_of::<u32>()) as BufferAddress);
    slice.map_async(MapMode::Read).await?;
    let map = slice.get_mapped_range();
    let bad_output: Vec<u32> = bytemuck::cast_slice(map.as_ref()).to_vec();

    let good_values: Vec<_> = ((good_size / 2)..good_size)
        .into_iter()
        .map(|x| x as u32)
        .collect();
    let bad_values: Vec<_> = ((bad_size / 2)..bad_size)
        .into_iter()
        .map(|x| x as u32)
        .collect();
    //    assert_eq!(good_values, good_output);
    println!("good passed");
    assert_eq!(bad_values, bad_output);
    Ok(())
}
