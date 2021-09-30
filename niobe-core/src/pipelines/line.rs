use crate::buffer::{Buffer, BufferSlice};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec2;
use palette::rgb::Rgba;
use std::convert::TryInto;
use std::mem;
use std::num::NonZeroU64;
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BufferBinding, Device, RenderPass, RenderPipeline,
    ShaderLocation, ShaderModule, SurfaceConfiguration,
};

pub enum HasDynamicOffset {
    False,
    True,
}

impl Into<bool> for HasDynamicOffset {
    fn into(self) -> bool {
        self as u8 != 0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LineVertex {
    pub pos: Vec2,
}
unsafe impl Pod for LineVertex {}
unsafe impl Zeroable for LineVertex {}

#[repr(C, align(256))]
#[derive(Copy, Clone)]
pub struct LineUniform {
    pub color: Rgba,
    pub scale: Vec2,
    pub translate: Vec2,
    pub line_scale: Vec2,
}

unsafe impl Pod for LineUniform {}
unsafe impl Zeroable for LineUniform {}

pub struct LineStorage {
    segment_vbo: wgpu::Buffer,
    shader: ShaderModule,
    pub uniforms: Vec<LineUniform>,
}

impl LineStorage {
    pub fn new(device: &Device) -> Self {
        const SEGMENT: [[f32; 2]; 6] = [
            [0.0f32, -0.5],
            [1., -0.5],
            [1., 0.5],
            [0., -0.5],
            [1., 0.5],
            [0., 0.5],
        ];

        let segment_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("line segment vbo"),
            contents: bytemuck::cast_slice(&SEGMENT),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("line shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
        });
        Self {
            segment_vbo,
            shader,
            uniforms: vec![],
        }
    }
}

pub struct LineGroup {
    bind_group: BindGroup,
}

impl LineGroup {
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

pub struct LinePipeline<'s> {
    pipeline: wgpu::RenderPipeline,
    line_storage: &'s LineStorage,
}

impl<'s> LinePipeline<'s> {
    const VERTEX_LOCATION: ShaderLocation = 0;
    const INSTANCE_LOCATION: ShaderLocation = 1;
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        line_storage: &'s LineStorage,
        has_dynamic_offset: HasDynamicOffset,
    ) -> Self {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("line render pipeline layout"),
                bind_group_layouts: &[&LineGroup::layout(device)],
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
                        array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
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
                front_face: wgpu::FrontFace::Cw,
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
            line_storage,
        }
    }

    pub fn group_drawer<'e, E: RenderEncoder<'e>>(
        &'e self,
        encoder: &'e mut E,
    ) -> LineGroupDrawer<'e, E> {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_vertex_buffer(0, self.line_storage.segment_vbo.slice(..));
        LineGroupDrawer { encoder: encoder }
    }
}
pub struct LineGroupDrawer<'e, E> {
    encoder: &'e mut E,
}

impl<'e, E: RenderEncoder<'e>> LineGroupDrawer<'e, E> {
    pub fn drawer(
        &'e mut self,
        group: &'e LineGroup,
        line_ubo: &BufferSlice<'e, LineUniform>,
    ) -> LineDrawer<'e, E> {
        self.encoder.set_bind_group(
            0,
            &group.bind_group,
            &[line_ubo.raw_addres_range().start.try_into().unwrap()],
        );
        self.encoder.set_vertex_buffer(0, line_ubo.to_raw_slice());
        LineDrawer { group: self }
    }
}

pub struct LineDrawer<'e, E> {
    group: &'e mut LineGroupDrawer<'e, E>,
}

impl<'e, E: RenderEncoder<'e>> LineDrawer<'e, E> {
    pub fn draw(&mut self, vertices: &BufferSlice<'e, LineVertex>) {
        self.group
            .encoder
            .set_vertex_buffer(1, vertices.to_raw_slice());
        self.group
            .encoder
            .set_vertex_buffer(2, vertices.to_raw_slice());
        self.group.encoder.draw(0..6 as _, vertices.range());
    }
}
