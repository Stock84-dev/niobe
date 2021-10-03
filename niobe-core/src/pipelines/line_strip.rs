use crate::buffer::{Buffer, BufferSlice};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec2;
use rgb::RGBA;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroU64;
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BufferBinding, Device, DynamicOffset, RenderPass,
    RenderPipeline, ShaderLocation, ShaderModule, SurfaceConfiguration,
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LineVertex {
    pub pos: Vec2,
}

unsafe impl Pod for LineVertex {}
unsafe impl Zeroable for LineVertex {}

impl LineVertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
        }
    }
}

#[repr(C, align(256))]
#[derive(Copy, Clone, Debug)]
pub struct LineUniform {
    pub color: RGBA<f32>,
    pub scale: Vec2,
    pub translate: Vec2,
    pub line_scale: Vec2,
}

unsafe impl Pod for LineUniform {}
unsafe impl Zeroable for LineUniform {}

pub struct LineShader {
    shader: ShaderModule,
}

impl LineShader {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("line shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
        });
        Self { shader }
    }
}

pub struct LineBindGroup {
    bind_group: BindGroup,
}

impl LineBindGroup {
    pub fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("line pipeline uniform bind group"),
        })
    }

    pub fn new(device: &Device, slice: &BufferSlice<'_, LineUniform>) -> Self {
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &slice.buf(),
                    offset: slice.raw_addres_range().start,
                    size: Some(NonZeroU64::new(std::mem::size_of::<LineUniform>() as _).unwrap()),
                }),
            }],
            label: Some("uniform line group"),
        });
        Self {
            bind_group: uniform_bind_group,
        }
    }
}

pub struct LinePipeline {
    pipeline: wgpu::RenderPipeline,
    segment_vbo: wgpu::Buffer,
}

impl LinePipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration, line_storage: &LineShader) -> Self {
        let segment_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("line segment vbo"),
            contents: bytemuck::cast_slice(&SEGMENT),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("line render pipeline layout"),
                bind_group_layouts: &[&LineBindGroup::layout(device)],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_storage.shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<LineVertex>() as wgpu::BufferAddress * 2,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<LineVertex>() as wgpu::BufferAddress * 2,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &line_storage.shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });
        Self {
            pipeline: render_pipeline,
            segment_vbo,
        }
    }

    pub fn drawer<'s, 'e, E: RenderEncoder<'s>>(
        &'s self,
        encoder: &'e mut E,
    ) -> LineGroupDrawer<'s, 'e, E> {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_vertex_buffer(0, self.segment_vbo.slice(..));
        LineGroupDrawer {
            encoder,
            pipeline: self,
        }
    }
}

pub struct LineGroupDrawer<'s, 'e, E> {
    encoder: &'e mut E,
    pipeline: &'s LinePipeline,
}

impl<'s, 'e, E: RenderEncoder<'s>> LineGroupDrawer<'s, 'e, E> {
    pub fn bind_group(
        self,
        bind_group: &'s LineBindGroup,
        uniform_id: u32,
    ) -> LineDrawer<'s, 'e, E> {
        self.encoder.set_bind_group(
            0,
            &bind_group.bind_group,
            &[uniform_id * std::mem::size_of::<LineUniform>() as DynamicOffset],
        );
        LineDrawer {
            pipeline: self.pipeline,
            encoder: self.encoder,
        }
    }

    pub fn finish(self) -> &'s LinePipeline {
        self.pipeline
    }
}

pub struct LineDrawer<'s, 'e, E> {
    pipeline: &'s LinePipeline,
    encoder: &'e mut E,
}

impl<'s, 'e, E: RenderEncoder<'s>> LineDrawer<'s, 'e, E> {
    pub fn draw(self, vertices: BufferSlice<'s, LineVertex>) -> Self {
        self.encoder.set_vertex_buffer(1, vertices.to_raw_slice());
        self.encoder.set_vertex_buffer(2, vertices.to_raw_slice());
        let mut range = vertices.range();
        range.end /= 2;
        self.encoder.draw(0..6 as _, range);
        self
    }

    pub fn finish(self) -> LineGroupDrawer<'s, 'e, E> {
        LineGroupDrawer {
            encoder: self.encoder,
            pipeline: self.pipeline,
        }
    }
}

const SEGMENT: [[f32; 2]; 6] = [
    [0.0f32, -0.5],
    [1., -0.5],
    [1., 0.5],
    [0., -0.5],
    [1., 0.5],
    [0., 0.5],
];
