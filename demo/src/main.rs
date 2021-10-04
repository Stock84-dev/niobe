// good tutuorials for graphs: https://en.wikibooks.org/wiki/OpenGL_Programming
//use crate::compute::Compute;
use crate::instance_shader::InstanceShader;
use anyhow::Result;
use cgmath::{ortho, vec2, InnerSpace, Matrix4, Ortho, Rotation3, Vector2, Zero};
use futures::executor::{LocalPool, LocalSpawner};
use futures::task::SpawnExt;
use futures::StreamExt;
use glyph_brush::ab_glyph::PxScale;
use glyph_brush::{HorizontalAlign, Layout, OwnedSection, OwnedText, VerticalAlign};
use lexical::write_float_options::RoundMode;
use log::*;
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
use nalgebra_glm::Vec2;
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{
    LineBindGroup, LinePipeline, LineShader, LineStripPipeline, LineUniform,
};
use niobe_core::pipelines::mesh::{MeshBindGroup, MeshPipeline, MeshShader, MeshUniform};
use niobe_core::Point2dExt;
use niobe_core::{colors, Mesh2d, Point2d};
use palette::rgb::Rgba;
use pipe_trait::*;
use rgb::RGBA;
use std::convert::TryInto;
use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::Rem;
use std::time::Instant;
use std::{iter, mem};
use wgpu::util::{DeviceExt, StagingBelt};
use wgpu::BufferBinding;
use wgpu::{
    BindingResource, BufferSize, BufferUsages, Color, DynamicOffset, Maintain, MapMode,
    TextureFormat,
};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, Section, Text};
use winit::dpi::PhysicalPosition;
use winit::window::Window;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod compute;
mod line_shader;
//mod lion;
mod bug;
mod bug2;
mod bug3;
mod instance_shader;
mod texture;

const BORDER_ID: usize = 0;
const GRID_ID: usize = 1;
const CROSSHAIR_ID: usize = 2;
const LINES_ID: usize = 3;
const BORDER_COLOR: RGBA<f32> = RGBA {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.,
};
const GRID_COLOR: RGBA<f32> = RGBA {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.,
};
const BACKGROUND_COLOR: RGBA<f32> = RGBA {
    r: 0.01,
    g: 0.01,
    b: 0.01,
    a: 1.,
};

const N_BORDER_VERTICES: usize = 5;
const N_CROSSHAIR_VERTICES: usize = 4;
const N_UI_VERTICES: usize = N_BORDER_VERTICES + N_CROSSHAIR_VERTICES;

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
    //    bug2::main()?;
    //    return Ok(());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    // State::new uses async code, so we're going to wait for it to finish
    //    let mut compute = pollster::block_on(compute::Compute::new(&window))?;
    //    pollster::block_on(compute.compute())?;
    //    return Ok(());
    let mut state = pollster::block_on(State::new(&window))?;
    let mut left_hold = false;
    let mut mouse_start_pos = Vec2::new(0., 0.);
    let mut mouse_pos = Vec2::new(0., 0.);

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
                            let pos = Vec2::new(position.x as f32, position.y as f32);
                            let delta = pos - mouse_pos;
                            mouse_pos = pos;
                            state.mouse_moved(mouse_pos);
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
    line_group: LineBindGroup,
    line_pipeline: LinePipeline,
    line_strip_pipeline: LineStripPipeline,
    mesh_pipeline: MeshPipeline,
    line_vbo: Buffer<Point2d>,
    ui_lines: Vec<Point2d>,
    uniforms: Vec<LineUniform>,
    ui_vbo: Buffer<Point2d>,
    line_ubo: Buffer<LineUniform>,
    instance_shader: InstanceShader,
    points: Vec<Point2d>,
    margin: u32,
    n_x_ticks: usize,
    n_y_ticks: usize,
    x_sections: Vec<OwnedSection>,
    y_sections: Vec<OwnedSection>,
    crosshair_sections: Vec<OwnedSection>,
    glyph_brush: GlyphBrush<()>,
    staging_belt: StagingBelt,
    pool: LocalPool,
    spawner: LocalSpawner,
    border_width: u32,
    grid_line_width: u32,
    crosshair_scale_hover_vbo: Buffer<Point2d>,
    crosshair_scale_hover_ibo: Buffer<u16>,
    mesh_ubo: Buffer<MeshUniform>,
    mesh_uniforms: Vec<MeshUniform>,
    mesh_group: MeshBindGroup,
    mesh_instance_vbo: Buffer<Point2d>,
    mesh_instance_positions: Vec<Point2d>,
    x_hover_width: u32,
    y_hover_height: u32,
    crosshair_scale_hover_vertices: Vec<Point2d>,
    mouse_pos: Vec2,
}

impl State {
    fn update_ticks(&mut self) {
        let pixel_scale = Vec2::new(
            1. / self.size.width as f32 * 2.,
            1. / self.size.height as f32 * 2.,
        );
        const FORMAT: u128 = lexical::format::STANDARD;
        let options = lexical::WriteFloatOptions::builder()
            // Only write up to 5 significant digits, IE, `1.23456` becomes `1.2345`.
            .max_significant_digits(NonZeroUsize::new(2))
            .round_mode(RoundMode::Round)
            // Trim the trailing `.0` from integral float strings.
            .trim_floats(true)
            .decimal_point(b'.')
            .build()
            .unwrap();

        let uni = &self.uniforms[LINES_ID];
        let mut log = 10.0f32.powf(-uni.scale.x.log10().floor()) / 100.;
        let start = (-1. - uni.translate.x + self.margin as f32 * pixel_scale.x) / uni.scale.x;
        let mut x_value = if start > 0. {
            start - start.rem(log) + log
        } else {
            start - start.rem(log)
        };
        let mut x = x_value * uni.scale.x + uni.translate.x;
        let mut tick_spacing = log * uni.scale.x;
        if 2. / tick_spacing > 50. {
            log *= 10.;
            tick_spacing *= 10.;
            x_value = start - start.rem(log);
            x = x_value * uni.scale.x + uni.translate.x;
        }

        let n_x_ticks = 2. / tick_spacing;
        for i in 0..self.n_x_ticks {
            self.x_sections[i].bounds =
                (self.size.width as f32 / n_x_ticks, self.size.height as f32);
            self.x_sections[i].screen_position = ((x + 1.) / pixel_scale.x, 1.95 / pixel_scale.y);
            self.x_sections[i].text[0].text =
                lexical::to_string_with_options::<_, FORMAT>(x_value, &options);
            self.ui_lines[N_UI_VERTICES + i * 2] = Point2d::new(x, -1.);
            self.ui_lines[N_UI_VERTICES + i * 2 + 1] = Point2d::new(x, 1.);
            x += tick_spacing;
            if x > 1. - self.margin as f32 * pixel_scale.x {
                for (j, section) in self.x_sections.iter_mut().enumerate().skip(i + 1) {
                    section.text[0].text = "".into();
                    self.ui_lines[N_UI_VERTICES + j * 2] = Point2d::new(0., 0.);
                    self.ui_lines[N_UI_VERTICES + j * 2 + 1] = Point2d::new(0., 0.);
                }
                break;
            }
            x_value += log;
        }

        let mut log = 10.0f32.powf(-uni.scale.y.log10().floor()) / 100.;
        let start = (-1. - uni.translate.y + self.margin as f32 * pixel_scale.y) / uni.scale.y;
        let mut y_value = if start > 0. {
            start - start.rem(log) + log
        } else {
            start - start.rem(log)
        };
        let mut y = y_value * uni.scale.y + uni.translate.y;
        let mut tick_spacing = log * uni.scale.y;
        if 2. / tick_spacing > 50. {
            log *= 10.;
            tick_spacing *= 10.;
            y_value = start - start.rem(log);
            y = y_value * uni.scale.y + uni.translate.y;
        }

        let n_y_ticks = 2. / tick_spacing;
        for i in 0..self.n_y_ticks {
            self.y_sections[i].bounds =
                (self.size.width as f32 / n_y_ticks, self.size.height as f32);
            self.y_sections[i].screen_position =
                (0., self.size.height as f32 - (y + 1.) / pixel_scale.y);
            self.y_sections[i].text[0].text =
                lexical::to_string_with_options::<_, FORMAT>(y_value, &options);
            self.ui_lines[N_UI_VERTICES + self.n_x_ticks * 2 + i * 2] = Point2d::new(-1., y);
            self.ui_lines[N_UI_VERTICES + self.n_x_ticks * 2 + i * 2 + 1] = Point2d::new(1., y);
            y += tick_spacing;
            if y > 1. - self.margin as f32 * pixel_scale.y {
                for (j, section) in self.y_sections.iter_mut().enumerate().skip(i + 1) {
                    section.text[0].text = "".into();
                    self.ui_lines[N_UI_VERTICES + self.n_x_ticks * 2 + j * 2] =
                        Point2d::new(0., 0.);
                    self.ui_lines[N_UI_VERTICES + self.n_x_ticks * 2 + j * 2 + 1] =
                        Point2d::new(0., 0.);
                }
                break;
            }
            y_value += log;
        }

        self.ui_vbo
            .write_sliced(&self.queue, N_UI_VERTICES.., &self.ui_lines);
    }

    fn mouse_moved(&mut self, mut pos: Vec2) {
        let pixel_scale = Vec2::new(
            1. / self.size.width as f32 * 2.,
            1. / self.size.height as f32 * 2.,
        );
        pos.x = pos.x * 2. / self.size.width as f32 - 1.;
        pos.y = (self.size.height as f32 - pos.y) * 2. / self.size.height as f32 - 1.;
        self.mouse_pos = pos;
        // horizontal line
        self.ui_lines[N_BORDER_VERTICES].x = -1.;
        self.ui_lines[N_BORDER_VERTICES].y = pos.y;
        self.ui_lines[N_BORDER_VERTICES + 1].x = 1.;
        self.ui_lines[N_BORDER_VERTICES + 1].y = pos.y;
        // vertical line
        self.ui_lines[N_BORDER_VERTICES + 2].x = pos.x;
        self.ui_lines[N_BORDER_VERTICES + 2].y = -1.;
        self.ui_lines[N_BORDER_VERTICES + 3].x = pos.x;
        self.ui_lines[N_BORDER_VERTICES + 3].y = 1.;
        self.mesh_uniforms[0].translate = pos;
        self.mesh_instance_positions[0].x = pos.x;
        self.mesh_instance_positions[1].y = pos.y;
        self.ui_vbo
            .write_sliced(&self.queue, N_BORDER_VERTICES.., &self.ui_lines);
        self.mesh_instance_vbo
            .write_sliced(&self.queue, .., &self.mesh_instance_positions);
        self.update_crosshair_scale_values();
    }

    fn update_crosshair_scale_values(&mut self) {
        let pixel_scale = Vec2::new(
            1. / self.size.width as f32 * 2.,
            1. / self.size.height as f32 * 2.,
        );
        let uni = &self.uniforms[LINES_ID];
        const FORMAT: u128 = lexical::format::STANDARD;
        let options = lexical::WriteFloatOptions::builder()
            // Only write up to 5 significant digits, IE, `1.23456` becomes `1.2345`.
            .max_significant_digits(NonZeroUsize::new(2))
            .round_mode(RoundMode::Round)
            // Trim the trailing `.0` from integral float strings.
            .trim_floats(true)
            .decimal_point(b'.')
            .build()
            .unwrap();
        let value = (self.mouse_pos - uni.translate).component_div(&uni.scale);
        self.crosshair_sections[0].screen_position = (
            (self.mouse_pos.x + 1.) / pixel_scale.x,
            1.95 / pixel_scale.y,
        );
        self.crosshair_sections[0].text[0].text =
            lexical::to_string_with_options::<_, FORMAT>(value.x, &options);
        self.crosshair_sections[1].screen_position = (
            0.,
            self.size.height as f32 - (self.mouse_pos.y + 1.) / pixel_scale.y,
        );
        self.crosshair_sections[1].text[0].text =
            lexical::to_string_with_options::<_, FORMAT>(value.y, &options);
    }

    fn zoom(&mut self, mut delta: f32) {
        println!("input zoom {:?}", delta);
        let base = 0.05;
        if delta.is_sign_positive() {
            delta += base;
        } else {
            delta = 1. - delta.abs() * base;
        }
        println!("zoom {:?}", delta);
        self.uniforms.iter_mut().skip(LINES_ID).for_each(|x| {
            x.line_scale *= 2. - delta;
            x.scale *= delta
        });
        self.update_ticks();
        self.line_ubo
            .write_sliced(&self.queue, LINES_ID.., &self.uniforms);
        self.update_crosshair_scale_values();
    }

    fn pan(&mut self, mut physical_delta: Vec2) {
        println!("pan {:?}", physical_delta);
        println!("size {:?}", self.size);
        physical_delta.x /= self.size.width as f32;
        physical_delta.y /= -(self.size.height as f32); // flip direction
        let delta = physical_delta;

        self.uniforms
            .iter_mut()
            .skip(LINES_ID)
            .for_each(|x| x.translate += delta * 2.);
        self.update_ticks();
        self.line_ubo
            .write_sliced(&self.queue, LINES_ID.., &self.uniforms);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > self.margin * 2 && new_size.height > self.margin * 2 {
            self.size = new_size;
            let pixel_scale = Vec2::new(
                1. / self.size.width as f32 * 2.,
                1. / self.size.height as f32 * 2.,
            );
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.uniforms[BORDER_ID].scale = Vec2::new(0.9, 0.9);
            let scale = Vec2::new(
                (self.size.width - self.margin * 2) as f32 / self.size.width as f32,
                (self.size.height - self.margin * 2) as f32 / self.size.height as f32,
            );
            self.uniforms[BORDER_ID].scale = scale;
            self.uniforms[BORDER_ID].line_scale.x = self.border_width as f32 * pixel_scale.x;
            self.uniforms[BORDER_ID].line_scale.y = self.border_width as f32 * pixel_scale.y;
            self.uniforms[GRID_ID].line_scale.x = self.grid_line_width as f32 * pixel_scale.x;
            self.uniforms[GRID_ID].line_scale.y = self.grid_line_width as f32 * pixel_scale.y;
            self.line_ubo
                .write(&self.queue, 0, &self.uniforms[..GRID_ID + 1]);
            println!(
                "scale {:?} width {:?}",
                self.uniforms[BORDER_ID].scale, self.uniforms[BORDER_ID].line_scale
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update_bg_hover(&mut self) {
        let pixel_scale = Vec2::new(
            1. / self.size.width as f32 * 2.,
            1. / self.size.height as f32 * 2.,
        );
        let x_hover_width = self.x_hover_width as f32 * pixel_scale.x / 2.;
        let y_hover_height = self.y_hover_height as f32 * pixel_scale.y / 2.;
        self.crosshair_scale_hover_vertices[0].x = -x_hover_width;
        self.crosshair_scale_hover_vertices[1].x = x_hover_width;
        self.crosshair_scale_hover_vertices[2].x = x_hover_width;
        self.crosshair_scale_hover_vertices[2].y = -1. + self.margin as f32 * pixel_scale.y;
        self.crosshair_scale_hover_vertices[3].x = -x_hover_width;
        self.crosshair_scale_hover_vertices[3].y = -1. + self.margin as f32 * pixel_scale.y;

        self.crosshair_scale_hover_vertices[4].y = -y_hover_height;
        self.crosshair_scale_hover_vertices[5].x = -1. + self.margin as f32 * pixel_scale.x;
        self.crosshair_scale_hover_vertices[5].y = -y_hover_height;
        self.crosshair_scale_hover_vertices[6].x = -1. + self.margin as f32 * pixel_scale.x;
        self.crosshair_scale_hover_vertices[6].y = y_hover_height;
        self.crosshair_scale_hover_vertices[7].y = y_hover_height;
        self.crosshair_scale_hover_vbo.write_sliced(
            &self.queue,
            ..,
            &self.crosshair_scale_hover_vertices,
        );
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder0 = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut render_pass0 = encoder0.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: BACKGROUND_COLOR.r as f64,
                        g: BACKGROUND_COLOR.g as f64,
                        b: BACKGROUND_COLOR.b as f64,
                        a: BACKGROUND_COLOR.a as f64,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        let tick_size = 5;
        let x_offset = self.margin - tick_size;
        let y_offset = self.margin - tick_size;
        self.line_strip_pipeline
            .drawer(&mut render_pass0)
            .bind_group(&self.line_group, BORDER_ID as u32)
            .draw(self.ui_vbo.slice(..N_BORDER_VERTICES as u32))
            .finish()
            .pipe(|mut x| {
                x.as_mut().set_scissor_rect(
                    self.margin + self.border_width / 2,
                    self.margin + self.border_width / 2,
                    self.size.width - self.margin * 2 - self.border_width,
                    self.size.height - self.margin * 2 - self.border_width,
                );
                x
            })
            .bind_group(&self.line_group, LINES_ID as u32)
            .draw(self.line_vbo.slice(..));

        self.line_pipeline
            .drawer(&mut render_pass0)
            .bind_group(&self.line_group, GRID_ID as u32)
            .draw(
                self.ui_vbo
                    .slice(N_BORDER_VERTICES as u32 + N_CROSSHAIR_VERTICES as u32..),
            )
            .finish()
            // draw crosshair last to be on top
            // TODO: set crosshair line width in pixels and scale it when window size changes
            .bind_group(&self.line_group, CROSSHAIR_ID as u32)
            .draw(self.ui_vbo.slice(
                N_BORDER_VERTICES as u32..N_BORDER_VERTICES as u32 + N_CROSSHAIR_VERTICES as u32,
            ));
        for section in &self.x_sections {
            self.glyph_brush.queue(section);
        }
        for section in &self.y_sections {
            self.glyph_brush.queue(section);
        }

        drop(render_pass0);
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder0,
                &view,
                self.size.width,
                self.size.height,
            )
            .expect("Draw queued");

        let mut encoder1 = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder 1"),
            });
        let mut render_pass1 = encoder1.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass 1"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        let rect = self
            .glyph_brush
            .glyph_bounds(&self.crosshair_sections[0])
            .unwrap();
        self.x_hover_width = rect.width() as u32 + 10;
        self.update_bg_hover();
        self.mesh_pipeline
            .drawer(
                &mut render_pass1,
                self.crosshair_scale_hover_vbo.slice(..4),
                self.crosshair_scale_hover_ibo.slice(..),
                &self.mesh_group,
                0,
            )
            .draw(self.mesh_instance_vbo.slice(..1))
            .set_vertices(self.crosshair_scale_hover_vbo.slice(4..))
            .draw(self.mesh_instance_vbo.slice(1..2));
        for section in &self.crosshair_sections {
            self.glyph_brush.queue(section);
        }
        drop(render_pass1);
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder1,
                &view,
                self.size.width,
                self.size.height,
            )
            .expect("Draw queued");
        // must finish before submiting
        self.staging_belt.finish();
        self.queue.submit([encoder0.finish(), encoder1.finish()]);
        self.spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.pool.run_until_stalled();
        Ok(())
    }

    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let margin = 50;
        let pixel_scale = Vec2::new(1. / size.width as f32 * 2., 1. / size.height as f32 * 2.);
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
        let instance = wgpu::Instance::new(wgpu::Backends::GL);
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
        let n_ticks = 21;
        let tick_vertices: Vec<_> = (0..n_ticks)
            .into_iter()
            .map(|x| (x as f32 / n_ticks as f32) * range + start)
            .collect();
        let scale = Vec2::new(
            (size.width - margin * 2) as f32 / size.width as f32,
            (size.height - margin * 2) as f32 / size.height as f32,
        );

        let uniforms = vec![
            LineUniform {
                color: BORDER_COLOR,
                scale,
                translate: Vec2::new(0.0, 0.0),
                line_scale: Vec2::new(0.01, 0.01),
            },
            LineUniform {
                color: GRID_COLOR,
                scale: Vec2::new(1., 1.),
                translate: Vec2::new(0.0, 0.0),
                line_scale: Vec2::new(0.002, 0.002),
            },
            LineUniform {
                color: colors::RED,
                scale: Vec2::new(1., 1.),
                translate: Vec2::new(0.0, 0.0),
                line_scale: Vec2::new(0.002, 0.002),
            },
            LineUniform {
                color: colors::ORANGE,
                scale: Vec2::new(1., 1.),
                translate: Vec2::new(0.0, 0.0),
                line_scale: Vec2::new(0.002, 0.002),
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
        let mut ui_lines = vec![
            Point2d::new(-1.0f32, -1.0),
            Point2d::new(1.0, -1.0),
            Point2d::new(1.0, 1.0),
            Point2d::new(-1.0, 1.0),
            Point2d::new(-1.0, -1.0),
            // crosshair
            Point2d::new(-1.0, -1.0),
            Point2d::new(-1.0, -1.0),
            Point2d::new(-1.0, -1.0),
            Point2d::new(-1.0, -1.0),
        ];

        let n_x_ticks = 201;
        let x_step = (size.width - margin * 2) as f32 * pixel_scale.x / (n_x_ticks - 1) as f32;
        let mut x_tick_x_pos = -1. + margin as f32 * pixel_scale.x;
        let x_tick_y_pos = -1. + margin as f32 * pixel_scale.y;
        let x_tick_y_size = 10. * pixel_scale.y;
        for i in 0..n_x_ticks {
            // TODO: draw indexed
            // we are duplicating some vertices here, but it is better to keep the same shader
            // for few hundred vertices
            ui_lines.push(Point2d::new(x_tick_x_pos, x_tick_y_pos));

            let mut end_pos = x_tick_y_pos - x_tick_y_size;
            if i % 2 == 0 {
                end_pos -= x_tick_y_size;
            }

            ui_lines.push(Point2d::new(x_tick_x_pos, end_pos));
            x_tick_x_pos += x_step;
        }
        let n_y_ticks = 201;
        let y_step = (size.height - margin * 2) as f32 * pixel_scale.y / (n_y_ticks - 1) as f32;
        let mut y_tick_y_pos = -1. + margin as f32 * pixel_scale.y;
        let y_tick_x_pos = -1. + margin as f32 * pixel_scale.x;
        let y_tick_x_size = 10. * pixel_scale.x;
        for i in 0..n_y_ticks {
            let mut end_pos = y_tick_x_pos - y_tick_x_size;
            if i % 2 == 0 {
                end_pos -= y_tick_x_size;
            }
            ui_lines.push(Point2d::new(end_pos, y_tick_y_pos));
            // we are duplicating some vertices here, but it is better to keep the same shader
            // for few hundred vertices
            ui_lines.push(Point2d::new(y_tick_x_pos, y_tick_y_pos));
            y_tick_y_pos += y_step;
        }

        let ui_vbo = Buffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            &ui_lines,
        );
        let line_ubo = Buffer::new(
            &device,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &uniforms,
        );
        let max = 7.;
        let n_points = 64;
        let points: Vec<_> = (0..n_points)
            .into_iter()
            .map(|x| {
                let sinx = x as f32 / n_points as f32 * max;
                let y = (sinx).sin();
                let x = (x as f32 / n_points as f32 - 0.5) * 2.;
                Point2d::new(x, y)
            })
            .collect();
        let line_vbo = Buffer::new(&device, BufferUsages::VERTEX, &points);
        let format = surface.get_preferred_format(&adapter).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            //            format: TextureFormat::Rgba8UnormSrgb,
            //            format: TextureFormat::Bgra8UnormSrgb,
            format,
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

        println!("{:?}", points);
        let line_strip_pipeline =
            LineStripPipeline::new(&device, &config, &LineShader::new(&device));
        let line_pipeline = LinePipeline::new(&device, &config, &LineShader::new(&device));
        //        println!("{:?}", geometry.indices);

        // Prepare glyph_brush
        let inconsolata =
            ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))?;

        let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, format);
        let pool = futures::executor::LocalPool::new();
        let mut x_sections: Vec<_> = (0..n_x_ticks)
            .into_iter()
            .map(|x| OwnedSection {
                screen_position: (0.0, 0.0),
                bounds: (100.0, 100.0),
                layout: Layout::default_single_line().h_align(HorizontalAlign::Center),
                text: vec![OwnedText::new("".to_string())
                    .with_scale(PxScale { x: 15.0, y: 15.0 })
                    .with_color(colors::DARK_GRAY)],
            })
            .collect();
        let y_sections = (0..n_y_ticks)
            .into_iter()
            .map(|x| OwnedSection {
                screen_position: (0.0, 0.0),
                bounds: (0.0, 0.0),
                layout: Layout::default_single_line().v_align(VerticalAlign::Center),
                text: vec![OwnedText::new("".to_string())
                    .with_scale(PxScale { x: 15.0, y: 15.0 })
                    .with_color(colors::DARK_GRAY)],
            })
            .collect();
        let crosshair_sections = vec![
            OwnedSection {
                screen_position: (0.0, 0.0),
                bounds: (100.0, 100.0),
                layout: Layout::default_single_line().h_align(HorizontalAlign::Center),
                text: vec![OwnedText::new("".to_string())
                    .with_scale(PxScale { x: 15.0, y: 15.0 })
                    .with_color(colors::DARK_GRAY)],
            },
            OwnedSection {
                screen_position: (0.0, 0.0),
                bounds: (100.0, 100.0),
                layout: Layout::default_single_line().v_align(VerticalAlign::Center),
                text: vec![OwnedText::new("".to_string())
                    .with_scale(PxScale { x: 15.0, y: 15.0 })
                    .with_color(colors::DARK_GRAY)],
            },
        ];
        let mesh_shader = MeshShader::new(&device);
        let mesh_pipeline = MeshPipeline::new(&device, &config, &mesh_shader);
        let x_hover_width_px = 20;
        let y_hover_height_px = 15 + 10;
        let x_hover_width = x_hover_width_px as f32 * pixel_scale.x;
        let y_hover_height = y_hover_height_px as f32 * pixel_scale.y;
        let crosshair_scale_hover_vertices = vec![
            // x
            Point2d::new(0., -1.),
            Point2d::new(x_hover_width, -1.),
            Point2d::new(x_hover_width, -1. + margin as f32 * pixel_scale.y),
            Point2d::new(0., -1. + margin as f32 * pixel_scale.y),
            // y
            Point2d::new(-1., 0.),
            Point2d::new(-1. + margin as f32 * pixel_scale.x, 0.),
            Point2d::new(-1. + margin as f32 * pixel_scale.x, y_hover_height),
            Point2d::new(-1., y_hover_height),
        ];
        let crosshair_scale_hover_indices = vec![0u16, 1, 2, 2, 3, 0];
        let crosshair_scale_hover_vbo = Buffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            &crosshair_scale_hover_vertices,
        );
        let crosshair_scale_hover_ibo =
            Buffer::new(&device, BufferUsages::INDEX, &crosshair_scale_hover_indices);
        let mesh_uniforms = vec![MeshUniform {
            color: colors::BROWN,
            scale: Vec2::new(1., 1.),
            translate: Vec2::new(0., 0.),
            mesh_scale: Vec2::new(1., 1.),
        }];
        let mesh_ubo = Buffer::new(
            &device,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &mesh_uniforms,
        );
        let mesh_group = MeshBindGroup::new(&device, mesh_ubo.slice(..));
        let mesh_instance_positions = vec![Point2d::default(), Point2d::default()];
        let mesh_instance_vbo = Buffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            &mesh_instance_positions,
        );

        Ok(Self {
            glyph_brush,
            line_group: LineBindGroup::new(&device, &line_ubo.slice(..)),
            instance_shader: InstanceShader::new(&device, &config, HasDynamicOffset::False),
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            line_strip_pipeline,
            mesh_pipeline,
            line_vbo,
            ui_lines,
            uniforms,
            ui_vbo,
            line_ubo,
            margin,
            n_x_ticks,
            line_pipeline,
            n_y_ticks,
            x_sections,
            points,
            staging_belt: wgpu::util::StagingBelt::new(1024),
            spawner: pool.spawner(),
            border_width: 5,
            pool,
            y_sections,
            grid_line_width: 2,
            crosshair_scale_hover_vbo,
            crosshair_scale_hover_ibo,
            mesh_ubo,
            mesh_uniforms,
            mesh_group,
            mesh_instance_vbo,
            mesh_instance_positions,
            crosshair_scale_hover_vertices,
            x_hover_width: x_hover_width_px,
            y_hover_height: y_hover_height_px,
            crosshair_sections,
            mouse_pos: Default::default(),
        })
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
