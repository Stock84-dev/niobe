use crate::buffer::{Buffer, BufferSlice};
use crate::Point2d;
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec2;
use private::Sealed;
use rgb::RGBA;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroU64;
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BufferAddress, BufferBinding, Device,
    DynamicOffset, RenderPass, RenderPipeline, ShaderLocation, ShaderModule, SurfaceConfiguration,
    TextureFormat,
};

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
    pub fn new(device: &Device, format: TextureFormat, line_storage: &LineShader) -> Self {
        let pipeline = create_pipeline(&device, format, line_storage, Self::LINE_MULTIPLIER as u64);
        Self {
            pipeline: pipeline.1,
            segment_vbo: pipeline.0,
        }
    }

    pub fn drawer<'s, 'e, E: RenderEncoder<'s>>(
        &'s self,
        encoder: &'e mut E,
    ) -> LineDrawer<'s, 'e, E, Self> {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_vertex_buffer(0, self.segment_vbo.slice(..));
        LineDrawer {
            encoder,
            pipeline: self,
        }
    }
}

impl LineRenderer for LinePipeline {
    const LINE_MULTIPLIER: u32 = 2;
}

pub struct LineStripPipeline {
    pipeline: wgpu::RenderPipeline,
    segment_vbo: wgpu::Buffer,
}

impl LineStripPipeline {
    pub fn new(device: &Device, format: TextureFormat, line_storage: &LineShader) -> Self {
        let pipeline = create_pipeline(&device, format, line_storage, Self::LINE_MULTIPLIER as u64);
        Self {
            pipeline: pipeline.1,
            segment_vbo: pipeline.0,
        }
    }

    pub fn drawer<'s, 'e, E: RenderEncoder<'s>>(
        &'s self,
        encoder: &'e mut E,
    ) -> LineDrawer<'s, 'e, E, Self> {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_vertex_buffer(0, self.segment_vbo.slice(..));
        LineDrawer {
            encoder,
            pipeline: self,
        }
    }
}

impl LineRenderer for LineStripPipeline {
    const LINE_MULTIPLIER: u32 = 1;
}

#[derive(AsMut)]
pub struct LineGroupDrawer<'s, 'e, E, P> {
    #[as_mut]
    encoder: &'e mut E,
    pipeline: &'s P,
}

impl<'s, 'e, E: RenderEncoder<'s>, P: LineRenderer> LineGroupDrawer<'s, 'e, E, P> {
    pub fn finish(self) -> &'s P {
        self.pipeline
    }
}

#[derive(AsMut)]
pub struct LineDrawer<'s, 'e, E, P> {
    #[as_mut]
    pipeline: &'s P,
    encoder: &'e mut E,
}

impl<'s, 'e, E: RenderEncoder<'s>, P: LineRenderer> LineDrawer<'s, 'e, E, P> {
    pub fn set_bind_group(&mut self, bind_group: &'s LineBindGroup, uniform_id: u32) -> &mut Self {
        self.encoder.set_bind_group(
            0,
            &bind_group.bind_group,
            &[uniform_id * std::mem::size_of::<LineUniform>() as DynamicOffset],
        );
        self
    }

    pub fn draw(&mut self, vertices: BufferSlice<'s, Point2d>) -> &mut Self {
        self.encoder.set_vertex_buffer(1, vertices.to_raw_slice());
        self.encoder.set_vertex_buffer(2, vertices.to_raw_slice());
        let range = vertices.range();
        let mut count = range.end - range.start;
        if P::LINE_MULTIPLIER == 1 {
            count -= 1;
        } else {
            count /= P::LINE_MULTIPLIER;
        }
        // Since instance wertex buffers are sliced we start from 0
        self.encoder.draw(0..6 as _, 0..count);
        self
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

fn create_pipeline(
    device: &Device,
    format: TextureFormat,
    line_storage: &LineShader,
    stride_multiplier: BufferAddress,
) -> (wgpu::Buffer, RenderPipeline) {
    let segment_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("line segment vbo"),
        contents: bytemuck::cast_slice(&SEGMENT),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                    array_stride: std::mem::size_of::<Point2d>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Point2d>() as wgpu::BufferAddress
                        * stride_multiplier,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Point2d>() as wgpu::BufferAddress
                        * stride_multiplier,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        offset: std::mem::size_of::<Point2d>() as wgpu::BufferAddress,
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
                format,
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
    (segment_vbo, render_pipeline)
}

pub trait LineRenderer: Sealed {
    const LINE_MULTIPLIER: u32;
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::LinePipeline {}
    impl Sealed for super::LineStripPipeline {}
}
