use crate::buffer::{Buffer, BufferSlice};
use crate::{IndexFormat, Mesh2d, Point2d};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec2;
use rgb::RGBA;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroU64;
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BufferAddress, BufferBinding, Device,
    DynamicOffset, RenderPass, RenderPipeline, ShaderLocation, ShaderModule, SurfaceConfiguration,
};

#[repr(C, align(256))]
#[derive(Copy, Clone, Debug)]
pub struct MeshUniform {
    pub color: RGBA<f32>,
    pub scale: Vec2,
    pub translate: Vec2,
    pub mesh_scale: Vec2,
}

unsafe impl Pod for MeshUniform {}
unsafe impl Zeroable for MeshUniform {}

pub struct MeshShader {
    shader: ShaderModule,
}

impl MeshShader {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("mesh shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mesh.wgsl").into()),
        });
        Self { shader }
    }
}

pub struct MeshBindGroup {
    bind_group: BindGroup,
}

impl MeshBindGroup {
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
            label: Some("mesh uniform bind group"),
        })
    }

    pub fn new(device: &Device, slice: BufferSlice<'_, MeshUniform>) -> Self {
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &slice.buf(),
                    offset: slice.raw_addres_range().start,
                    size: Some(NonZeroU64::new(std::mem::size_of::<MeshUniform>() as _).unwrap()),
                }),
            }],
            label: Some("uniform mesh group"),
        });
        Self {
            bind_group: uniform_bind_group,
        }
    }
}

pub struct MeshPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl MeshPipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration, mesh_shader: &MeshShader) -> Self {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("line render pipeline layout"),
                bind_group_layouts: &[&MeshBindGroup::layout(device)],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader.shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Point2d>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Point2d>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader.shader,
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
        }
    }

    pub fn drawer<'s, 'e, E: RenderEncoder<'s>, IF: IndexFormat>(
        &'s self,
        encoder: &'e mut E,
    ) -> MeshDrawer<'e, E> {
        encoder.set_pipeline(&self.pipeline);
        MeshDrawer {
            encoder,
            vertices_len: 0,
            indices_len: 0,
        }
    }
}

#[derive(AsMut)]
pub struct MeshDrawer<'e, E> {
    #[as_mut]
    encoder: &'e mut E,
    vertices_len: u32,
    indices_len: u32,
}

impl<'s, 'e, E: RenderEncoder<'s>> MeshDrawer<'e, E> {
    pub fn set_vertices(&mut self, vertices: BufferSlice<'s, Point2d>) -> &mut Self {
        self.vertices_len = vertices.len();
        self.encoder.set_vertex_buffer(0, vertices.to_raw_slice());
        self
    }

    pub fn set_indices<IF: IndexFormat>(&mut self, indices: BufferSlice<'s, IF>) -> &mut Self {
        self.indices_len = indices.len();
        self.encoder
            .set_index_buffer(indices.to_raw_slice(), IF::FORMAT);
        self
    }

    pub fn set_bind_group(&mut self, bind_group: &'s MeshBindGroup, uniform_id: u32) -> &mut Self {
        self.encoder.set_bind_group(
            0,
            &bind_group.bind_group,
            &[uniform_id * std::mem::size_of::<MeshUniform>() as DynamicOffset],
        );
        self
    }

    pub fn draw(&mut self, instances: BufferSlice<'s, Point2d>) -> &mut Self {
        self.encoder.set_vertex_buffer(1, instances.to_raw_slice());
        // Since instance vertex buffers are sliced we start from 0
        self.encoder
            .draw_indexed(0..self.indices_len, 0, 0..instances.len());
        self
    }
}
