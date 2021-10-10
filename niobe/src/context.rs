use crate::combo::ChartCombo;
use crate::components::view::DrawControlFlow;
use crate::components::Component;
use epaint::emath::{Pos2, Rect, Vec2};
use epaint::Color32;
use niobe_core::pipelines::line::{LineShader, LineStripPipeline};
use niobe_core::pipelines::ui::UiRenderPass;
use niobe_core::pipelines::{Drawer, PipelineKind};
use std::sync::Arc;
use wgpu::{Color, Device, LoadOp, Queue, RenderPass, RenderPipeline, TextureFormat, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::{Window, WindowId};

struct Pipeline {
    pipeline: RenderPipeline,
    min_z: i8,
    max_z: i8,
    cur_z: i8,
}

pub struct DeviceContext {
    queue: Arc<Queue>,
    device: Arc<Device>,
    ui_render_pass: UiRenderPass,
    line_strip_pipeline: LineStripPipeline,
}

pub struct WindowContext {
    ui_render_pass: UiRenderPass,
    line_strip_pipeline: LineStripPipeline,
    components: Vec<Arc<dyn Component>>,
    combos: Vec<ChartCombo>,
    clear_color: Option<wgpu::Color>,
    window_size: PhysicalSize<u32>,
    scale_factor: f32,
    window_id: WindowId,
}

impl WindowContext {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        window: &Window,
        window_size: PhysicalSize<u32>,
        scale_factor: f32,
        clear_color: Option<Color32>,
    ) -> Self {
        let clear_color = clear_color.map(|x| wgpu::Color {
            r: x.r() as f64 / 255.,
            g: x.g() as f64 / 255.,
            b: x.b() as f64 / 255.,
            a: x.a() as f64 / 255.,
        });
        let line_shader = LineShader::new(&device);
        Self {
            ui_render_pass: UiRenderPass::new(device, format),
            line_strip_pipeline: LineStripPipeline::new(&device, format, &line_shader),
            combos: vec![ChartCombo::new()],
            clear_color,
            window_size: window.inner_size(),
            scale_factor: window.scale_factor() as f32,
            window_id: window.id(),
        }
    }

    pub fn draw(&self, device: &Device, queue: &Queue, view: &TextureView) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("niobe render encoder"),
        });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("niobe render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match self.clear_color {
                        None => LoadOp::Load,
                        Some(color) => LoadOp::Clear(color),
                    },
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        let mut drawer = Drawer {
            pipeline: &self.line_strip_pipeline,
            queue,
            pass: &render_pass,
        };
        loop {
            let mut control_flow = 0u8;
            drawer.pipeline = &self.line_strip_pipeline;
            for combo in &self.combos {
                control_flow |= combo.draw(&mut drawer) as u8;
            }
            debug_assert!(control_flow <= DrawControlFlow::DrawRequested as u8);
            let control_flow = unsafe { std::mem::transmute(control_flow) };
            if let DrawControlFlow::Finished = control_flow {
                break;
            }
        }
        let mut drawer = self.ui_render_pass.drawer(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.window_size.0 as f32, self.window_size.1 as f32),
        ));
        for combo in &self.combos {
            combo.draw_ui(&mut drawer);
        }
        self.ui_render_pass.render(
            device,
            queue,
            view,
            (self.window_size.width, self.window_size.height),
            self.scale_factor,
            self.clear_color,
        );
    }

    pub fn handle_event<T>(
        &mut self,
        event: &Event<T>,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
    ) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window_id => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        self.window_size = *physical_size;
                        return;
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        scale_factor,
                    } => {
                        self.scale_factor = scale_factor as f32;
                        self.window_size = **new_inner_size;
                        return;
                    }
                    _ => {}
                }
                for combo in &self.combos {
                    combo.input(event);
                }
            }
            Event::RedrawRequested(_) => {
                self.draw(device, queue, view);
                //                match state.render() {
                //                    Ok(_) => {}
                //                    // Reconfigure the surface if lost
                //                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                //                    // The system is out of memory, we should probably quit
                //                    Err(wgpu::SurfaceError::OutOfMemory) => {
                //                        eprintln!("out of memory");
                //                        *control_flow = ControlFlow::Exit
                //                    }
                //                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                //                    Err(e) => eprintln!("{:?}", e),
                //                }
            }
            _ => {}
        }
    }
}
