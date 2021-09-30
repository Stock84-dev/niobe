use std::sync::Arc;
use std::thread;
use wgpu::{BufferAddress, Maintain, MapMode};
const BUF_SIZE: BufferAddress = (std::mem::size_of::<f32>() * 8) as _;
const SOURCE: &'static str = "\
[[block]]
struct Data {
    data: array<f32>;
};
[[group(0), binding(0)]] var<storage, read_write> dest : Data;
[[stage(compute), workgroup_size(1)]]
fn main(
) {
    let val: f32 = 1.;
    dest.data[0] = val;
    dest.data[1] = val;
}
";

fn main() -> anyhow::Result<()> {
    env_logger::init();
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
            None, // Trace path
        )
        .await
        .unwrap();
    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    println!("{:#?}", adapter.get_info());
    let device = Arc::new(device);
    let dest_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: BUF_SIZE,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let d = device.clone();
    thread::spawn(move || loop {
        d.poll(Maintain::Wait);
    });

    let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
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

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
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

    let mut cpass =
        command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
    cpass.set_pipeline(&compute_pipeline);
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.dispatch(1 as _, 1, 1);
    drop(cpass);
    queue.submit(Some(command_encoder.finish()));
    let slice = dest_buffer.slice(..BUF_SIZE);
    slice.map_async(MapMode::Read).await?;
    let map = slice.get_mapped_range();
    let output: &[f32] = bytemuck::cast_slice(map.as_ref());
    println!("{:#?}", output);
    assert_eq!(output, [1.0f32, 1., 0., 0., 0., 0., 0., 0.]);
    Ok(())
}
