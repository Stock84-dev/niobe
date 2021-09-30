// good tutuorials for graphs: https://en.wikibooks.org/wiki/OpenGL_Programming
use anyhow::Result;
use wgpu::{
    BindingResource, BufferSize, BufferUsages, DynamicOffset, Maintain, MapMode, TextureFormat,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
const RED: [f32; 4] = [1.0, 0., 0., 1.];
const BLACK: [f32; 4] = [0.0, 0., 0., 1.];

mod compute;
mod line_shader;
mod niobe_state;
//mod lion;
mod bug;
mod bug2;
mod bug3;
mod instance_shader;
mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct XYInstance {
    point: [f32; 2],
}

impl XYInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<XYInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
    fn desc_inv<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<XYInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 8,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct XYVertex {
    position: [f32; 2],
}

impl XYVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<XYVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    normal: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    bug2::main()?;
    return Ok(());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    // State::new uses async code, so we're going to wait for it to finish
    //    let mut compute = pollster::block_on(compute::Compute::new(&window))?;
    //    pollster::block_on(compute.compute())?;
    //    return Ok(());
    let mut state = pollster::block_on(State::new(&window));
    let mut left_hold = false;
    let mut mouse_start_pos = Vector2::new(0., 0.);
    let mut mouse_pos = Vector2::new(0., 0.);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::MouseInput {
                            state: ElementState::Pressed,
                            button: MouseButton::Left,
                            ..
                        } => {
                            left_hold = true;
                        }
                        WindowEvent::MouseInput {
                            state: ElementState::Released,
                            button: MouseButton::Left,
                            ..
                        } => {
                            left_hold = false;
                        }
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(x, y),
                            ..
                        } => {
                            // delta is a vector of [0., +-1.]
                            state.zoom(*y);
                        }
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                            modifiers,
                        } => {
                            let pos = Vector2::new(position.x as f32, position.y as f32);
                            let delta = pos - mouse_pos;
                            mouse_pos = pos;
                            if left_hold {
                                state.pan(delta);
                            }
                        }
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("out of memory");
                        *control_flow = ControlFlow::Exit
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

//use crate::compute::Compute;
use crate::instance_shader::InstanceShader;
use cgmath::{ortho, vec2, InnerSpace, Matrix4, Ortho, Rotation3, Vector2, Zero};
use lyon::algorithms::math::{point, Point};
use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillVertexConstructor, LineJoin, StrokeVertexConstructor,
};
use lyon::tessellation;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::path::Path;
use lyon::tessellation::{
    FillTessellator, LineCap, StrokeOptions, StrokeTessellator, VertexBuffers,
};
use std::convert::TryInto;
use std::num::NonZeroU64;
use std::time::Instant;
use std::{iter, mem};
use wgpu::util::DeviceExt;
use wgpu::BufferBinding;
use winit::dpi::PhysicalPosition;
use winit::window::Window;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Copy, Clone, Debug)]
#[repr(C, align(256))]
struct Uniform {
    color: [f32; 4],
    scale: Vector2<f32>,
    translate: Vector2<f32>,
    line_scale: Vector2<f32>,
    ortho: Matrix4<f32>,
}

struct State {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    quad_vbo: wgpu::Buffer,
    border_ibo: wgpu::Buffer,
    instance_vbo: wgpu::Buffer,
    //    index_buffer: wgpu::Buffer,
    instances: Vec<[f32; 2]>,
    borders: Vec<Vector2<f32>>,
    //    vertices: Vec<Vertex>,
    //    indicies: Vec<u16>,
    uniforms: Vec<Uniform>,
    ubo: wgpu::Buffer,
    instance_shader: InstanceShader,
    uniform_bind_group: wgpu::BindGroup,
    margin: u32,
}

impl State {
    fn zoom(&mut self, mut delta: f32) {
        println!("input zoom {:?}", delta);
        let base = 0.05;
        if delta.is_sign_positive() {
            delta += base;
        } else {
            delta = 1. - delta.abs() * base;
        }
        println!("zoom {:?}", delta);
        self.uniforms
            .iter_mut()
            .skip(1)
            .for_each(|x| x.scale *= delta);
    }

    fn pan(&mut self, mut physical_delta: Vector2<f32>) {
        println!("pan {:?}", physical_delta);
        println!("size {:?}", self.size);
        physical_delta.x /= self.size.width as f32;
        physical_delta.y /= -(self.size.height as f32); // flip direction
        let delta = physical_delta;

        self.uniforms
            .iter_mut()
            .skip(1)
            .for_each(|x| x.translate += delta * 2.);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.uniforms[0].scale = vec2(0.9, 0.9);
            let scale = Vector2::new(
                (self.size.width - self.margin * 2) as f32 / self.size.width as f32,
                (self.size.height - self.margin * 2) as f32 / self.size.height as f32,
            );

            //            self.borders.iter_mut().for_each(|x| {
            //                x.x = x.x * scale.x;
            //                x.y = x.y * scale.y;
            //            });
            //            self.queue.write_buffer(&self.border_ibo, 0 as _, unsafe {
            //                std::slice::from_raw_parts(
            //                    self.borders.as_ptr() as *const u8,
            //                    self.borders.len() * std::mem::size_of::<Vector2<f32>>(),
            //                )
            //            });
            self.uniforms[0].scale = scale;
            //            self.uniforms[0].line_width = 0.01;
            let pixel_width = 10;
            //            self.uniforms[0].line_width = (pixel_width as f32 / self.size.width as f32);
            // we are scaling line width in shader so we are nautralizing here
            self.uniforms[0].line_scale.x =
                ((pixel_width as f32) / self.size.width as f32) / scale.x;
            self.uniforms[0].line_scale.y =
                ((pixel_width as f32) / self.size.height as f32) / scale.y;
            //                        self.uniforms[0].line_width = (pixel_width as f32 / self.size.width as f32)
            //                            .max(pixel_width as f32 / self.size.height as f32);
            println!(
                "scale {:?} width {:?}",
                self.uniforms[0].scale, self.uniforms[0].line_scale
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        unsafe {
            self.queue.write_buffer(
                &self.ubo,
                0 as _,
                std::slice::from_raw_parts(
                    self.uniforms.as_ptr() as *const u8,
                    self.uniforms.len() * 256,
                ),
            );
        }
        
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.quad_vbo.slice(..));

        // draw border
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[0]);
        render_pass.set_vertex_buffer(1, self.border_ibo.slice(..));
        render_pass.set_vertex_buffer(2, self.border_ibo.slice(..));
        render_pass.draw(0..6 as _, 0..(4) as _);

        //         draw plot
        render_pass.set_bind_group(
            0,
            &self.uniform_bind_group,
            &[std::mem::size_of::<Uniform>() as _],
        );
        render_pass.set_vertex_buffer(1, self.instance_vbo.slice(..));
        render_pass.set_vertex_buffer(2, self.instance_vbo.slice(..));
        render_pass.draw(0..6 as _, 0..(self.instances.len() - 1) as _);
        //        render_pass.draw(0..3 as _, 0..(1) as _);
        //        render_pass.draw(0..self.vertices.len() as _, 0..1);
        //        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        //        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //        render_pass.draw_indexed(0..self.indicies1.len() as _, 0, 0..1);
        //        render_pass.set_index_buffer(self.index_buffer2.slice(..), wgpu::IndexFormat::Uint32);
        //        render_pass.draw_indexed(0..6 as _, 0, 0..1);
        //        render_pass.draw_indexed(0..6 as _, 0, 0..1);
        //        render_pass.draw(0..4 as _, 0..(2) as _);

        drop(render_pass);
        let e = encoder.finish();
        let i = std::iter::once(e);
        // submit will accept anything that implements IntoIter
        self.queue.submit(i);
        Ok(())
    }

    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let margin = 50;
        //        let adapter = instance
        //        .enumerate_adapters(wgpu::Backends::all())
        //        .filter(|adapter| {
        //            // Check if this adapter supports our surface
        //            surface.get_preferred_format(&adapter).is_some()
        //        })
        //        .first()
        //        .unwrap();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        println!("surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        println!("{:#?}", adapter.get_info());
        println!("{:#?}", adapter.is_surface_supported(&surface));
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
        println!("device");
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Neighbor Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let start = 50.0f32;
        let end = 100.;
        let range = (end - start).abs();
        let n_ticks = 11;
        let tick_vertices: Vec<_> = (0..n_ticks)
            .into_iter()
            .map(|x| (x as f32 / n_ticks as f32) * range + start)
            .collect();
        let scale = Vector2::new(
            (size.width - margin * 2) as f32 / size.width as f32,
            (size.height - margin * 2) as f32 / size.height as f32,
        );

        let mut uniforms = vec![
            Uniform {
                color: [0.0, 0., 0., 1.],
                scale: vec2(1., 1.),
                translate: Vector2::new(0.0, 0.0),
                line_scale: Vector2::new(0.01, 0.01),
                ortho: ortho(0., size.width as f32, size.height as f32, 0., 0., 1.),
            },
            Uniform {
                color: [1.0, 0., 0., 1.],
                scale,
                translate: Vector2::new(0.0, 0.0),
                line_scale: Vector2::new(0.01, 0.01),
                ortho: ortho(0., size.width as f32, size.height as f32, 0., 0., 1.),
            },
        ];
        #[rustfmt::skip]
        let quad_vertices = [
             [0.0f32, -0.5],
             [1., -0.5],
             [1.,  0.5],
             [0., -0.5],
             [1.,  0.5],
             [0.,  0.5]
        ];
        let mut borders = vec![
            vec2(-1.0f32, -1.0),
            vec2(1.0, -1.0),
            vec2(1.0, 1.0),
            vec2(-1.0, 1.0),
            vec2(-1.0, -1.0),
        ];
        //
        //        borders.iter_mut().for_each(|x| {
        //            x.x *= 0.9;
        //            x.y *= 0.9;
        //        });

        let quad_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad Buffer"),
            contents: bytemuck::cast_slice(&[quad_vertices]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let border_ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("border Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    borders.as_ptr() as *const u8,
                    borders.len() * std::mem::size_of::<Vector2<f32>>(),
                )
            },
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(uniforms.as_ptr() as *const u8, uniforms.len() * 256)
            },
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | BufferUsages::MAP_READ,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                        //                        Some(BufferSize::new(256).unwrap()),
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(NonZeroU64::new(std::mem::size_of::<Uniform>() as _).unwrap()),
                }),
            }],
            label: Some("camera_bind_group"),
        });
        let max = 7.;
        let n_points = 64;
        let points: Vec<_> = (0..n_points)
            .into_iter()
            .map(|x| {
                let sinx = x as f32 / n_points as f32 * max;
                let y = (sinx).sin();
                let x = (x as f32 / n_points as f32 - 0.5) * 2.;
                [x, y]
            })
            .collect();
        let instance_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        //        let mut path_builder = Path::builder();
        //        path_builder.begin(point(0.0, 0.0));
        //        path_builder.line_to(point(1.0, 2.0));
        //        path_builder.line_to(point(2.0, 0.0));
        //        path_builder.line_to(point(1.0, 1.0));
        //        path_builder.end(true);
        //        let path = path_builder.build();
        //
        //        // Create the destination vertex and index buffers.
        //        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        //        let mut stroke_tess = StrokeTessellator::new();
        //        let mut fill_tess = FillTessellator::new();
        //        let options = StrokeOptions::default().with_line_width(0.01);
        //        stroke_tess
        //            .tessellate_path(
        //                &path,
        //                &options,
        //                &mut BuffersBuilder::new(&mut geometry, Ctor),
        //            )
        //            .unwrap();
        //        fill_tess
        //            .tessellate_path(
        //                &path,
        //                &FillOptions::tolerance(tolerance).with_fill_rule(tessellation::FillRule::NonZero),
        //                &mut BuffersBuilder::new(&mut geometry, Ctor),
        //            )
        //            .unwrap();

        //        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //            label: Some("Vertex Buffer"),
        //            contents: bytemuck::cast_slice(&geometry.vertices),
        //            usage: wgpu::BufferUsages::VERTEX,
        //        });
        //        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //            label: Some("Vertex Buffer"),
        //            contents: bytemuck::cast_slice(&geometry.indices),
        //            usage: wgpu::BufferUsages::INDEX,
        //        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            //            format: TextureFormat::Rgba8UnormSrgb,
            //            format: TextureFormat::Bgra8UnormSrgb,
            format: surface.get_preferred_format(&adapter).unwrap(),
            //            [Rgba8UnormSrgb, Bgra8UnormSrgb]
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        println!("configure");

        let now = Instant::now();
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        println!("{} ms", now.elapsed().as_millis());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[XYVertex::desc(), XYInstance::desc(), XYInstance::desc_inv()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });
        println!("{:?}", points);
        //        println!("{:?}", geometry.indices);

        Self {
            instance_shader: InstanceShader::new(&device, &config, HasDynamicOffset::False),
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            quad_vbo,
            //            index_buffer,
            instance_vbo,
            border_ibo,
            instances: points,
            //            vertices: geometry.vertices,
            //            indicies: geometry.indices,
            borders,
            uniforms: uniforms,
            ubo: uniform_buffer,
            uniform_bind_group,
            margin,
        }
    }
}

pub enum HasDynamicOffset {
    False,
    True,
}

impl Into<bool> for HasDynamicOffset {
    fn into(self) -> bool {
        self as u8 != 0
    }
}

/*
# core
TODO: to draw lines use RenderPipelineDescriptor.primitive::PrimitiveState.topology::PrimitiveTopology::LineList
Basic shapes that all live in one buffer, used for grid and ticks
I don't think that putting them all in one buffer is good idea, when a shape needs to be deallocated
then we need to shift all other buffers and indicies
Line:
width, dashed, color
Box
border color, fill color
Each shape has border color, border width and fill color
Texture
Text
Data Vertex buffer
- ring buffer
- reallocate
- set constant min/max for scaling or compute it based on contents that are showed, use compute shaders to find min/max
- generic, provide any data type, when calculating vertex position cast to f32 in shader
- scale matrix to smaller size than window
- transform matrix for position

CandleRenderer
1 vertex (OHLC)
6 indicies for candle body + 6 indicies for wicks
needs to be 1 vertex so that we can compute color based on O > C

ScatterPointRenderer
ScatterTextureRenderer
ScatterCircleRenderer
ScatterMeshRenderer
ScatterTextRenderer
ScatterLineRenderer
they all contain a vertex buffer that has generic x and y pos + other data

shared x axis
dont fit them all into one shader, gpu has limit on n bind buffers like 8, use multiple render
passes instead
i think we should use binding groups to bind 2 vertex buffers

LineRenderer
LineFillRenderer // fill color under line

ColumnRenderer
vertex: 2 * f32
indicies: 6
For vertical/horizontal column renderer we conditionally compile shader with transformation matrix

now repeat every renderer but add color to each vertex



# Higher abstraction layer:
plot from iter
crosshair
grid
border
ticks
display series name, color, current hover value in top right, when clicking on name hide it from plot
zoom, pan
double clicking on axis border resets scale
shift click -> measuring tool
subplots, define size and position
sub plots are connected, they have same x axis highlitght multiple crosshairs and sub plots can be
in different positions to not be one below another
log scale
circle with coordinates highlighted

# Even higher abstraction layer
- here is data, open window and plot it

how do you plot bollinger bands...with band renderer?




 */

struct Ctor;

impl StrokeVertexConstructor<Vertex> for Ctor {
    fn new_vertex(&mut self, mut vertex: tessellation::StrokeVertex) -> Vertex {
        //        println!("{:?}", vertex.position().to_array());
        //        println!("{:?}", vertex.normal().to_array());
        //        println!("{:?}", vertex.position_on_path().to_array());
        //        println!("{:?}", vertex.advancement());
        //        println!("{:?}", vertex.side());
        //        println!("{:?}", vertex.source());
        //        println!("{:?}", vertex.interpolated_attributes());
        let normal = vertex.normal();
        let mut pos = vertex.position();
        //        let mut pos = (vertex.position() + vertex.normal()) * 0.25;
        //        pos.x += normal.x;
        //        pos.y += normal.y;
        //        pos.x *= 0.25;
        //        pos.y *= 0.25;

        //        pos.x -= 1.;
        //        pos.y -= 1.;
        println!("{:?}", pos);
        Vertex {
            position: pos.to_array(),
            normal: vertex.normal().to_array(),
            color: rand::random(),
        }
    }
}

//impl FillVertexConstructor<Vertex> for Ctor {
//    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> Vertex {
//        Vertex {
//            position: vertex.position().to_array(),
//            normal: [0.0, 0.0],
//        }
//    }
//}
