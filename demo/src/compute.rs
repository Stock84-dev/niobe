use crate::XYVertex;
use anyhow::Result;
use std::convert::TryInto;
use std::sync::Arc;
use std::{mem, thread};
use wgpu::util::DeviceExt;
use wgpu::{BufferAddress, BufferDescriptor, BufferUsages, Maintain, MapMode};
use winit::window::Window;

pub struct Compute {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: wgpu::Buffer,
    dest_buffer: wgpu::Buffer,
    verticies: Vec<XYVertex>,
    bind_group: wgpu::BindGroup,
}

impl Compute {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
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
        println!("{:#?}", adapter.get_info());
        let device = Arc::new(device);
        let d = device.clone();
        thread::spawn(move || loop {
            d.poll(Maintain::Wait);
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Neighbor Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let verticies = vec![
            XYVertex {
                position: [0.0, 0.5],
            },
            XYVertex {
                position: [-0.5, -0.5],
            },
            XYVertex {
                position: [0.5, -0.5],
            },
        ];
        let max = 7.;
        let n_points = 32;
        let verticies: Vec<_> = (0..n_points)
            .into_iter()
            .map(|x| {
                let sinx = x as f32 / n_points as f32 * max;
                let y = (sinx).sin() + 1.;
                let x = (x as f32 / n_points as f32 - 0.5) * 2. + 1.;
                XYVertex { position: [x, y] }
            })
            .collect();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verticies),
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::STORAGE,
        });
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        struct Uniform {
            min_offset: u32,
            max_offset: u32,
            stride: u32,
            range_end_excluding: u32,
            dest_offset: u32,
        }
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&[Uniform {
                min_offset: 0,
                max_offset: 0,
                stride: 1,
                range_end_excluding: verticies.len() as u32 * 2,
                dest_offset: 0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let dest_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 256,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("minmax.wgsl").into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (mem::size_of::<Uniform>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (verticies.len() * mem::size_of::<XYVertex>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            //                            wgpu::BufferSize::new((8) as _),
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "minmax64",
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: dest_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        Ok(Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            compute_pipeline,
            vertex_buffer,
            dest_buffer,
            verticies,
            bind_group,
        })
    }

    pub async fn compute(&mut self) -> Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut cpass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.dispatch(1 as _, 1, 1);
        drop(cpass);
        self.queue.submit(Some(command_encoder.finish()));
        let slice = self.dest_buffer.slice(..256 as BufferAddress);
        slice.map_async(MapMode::Read).await?;
        let map = slice.get_mapped_range();
        let max: &[[f32; 2]] = bytemuck::cast_slice(map.as_ref());
        println!("{:?}", bytemuck::cast_slice::<_, [f32; 2]>(&self.verticies));
        println!("{:?}", max);
        Ok(())
    }
}
