pub mod scale;
//use anyhow::Result;
//use egui::epaint::text::Fonts;
//use egui::epaint::{Mesh, TessellationOptions, Tessellator};
//use egui::{ClippedMesh, Color32, FontDefinitions, Pos2, Rect, Shape, Stroke, TextStyle, Vec2};
//use egui_wgpu_backend::{epi, ScreenDescriptor};
//use std::iter;
//use winit::event::{
//    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
//};
//use winit::event_loop::{ControlFlow, EventLoop};
//use winit::window::{Window, WindowBuilder};
//
//struct EguiState {
//    instance: wgpu::Instance,
//    adapter: wgpu::Adapter,
//    surface: wgpu::Surface,
//    device: wgpu::Device,
//    queue: wgpu::Queue,
//    config: wgpu::SurfaceConfiguration,
//    egui_rpass: egui_wgpu_backend::RenderPass,
//    size: winit::dpi::PhysicalSize<u32>,
//    fonts: Fonts,
//    pos: f32,
//}
//
//impl EguiState {
//    fn mouse_moved(&mut self, mut pos: Vec2) {}
//
//    fn zoom(&mut self, mut delta: f32) {}
//
//    fn pan(&mut self, mut physical_delta: Vec2) {}
//
//    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
//        self.size = new_size;
//        self.config.width = new_size.width;
//        self.config.height = new_size.height;
//        self.surface.configure(&self.device, &self.config);
//    }
//
//    fn input(&mut self, event: &WindowEvent) -> bool {
//        false
//    }
//
//    fn update(&mut self) {}
//
//    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
//        self.pos += 1.;
//        let mut tessellator = Tessellator::from_options(TessellationOptions::default());
//        let mut mesh = Mesh::default();
//        let rect = Rect {
//            min: Pos2::new(0., 0.),
//            max: Pos2::new(0. + 100., 0. + 100.),
//        };
//        let rect2 = Rect {
//            min: Pos2::new(550., 550.),
//            max: Pos2::new(550. + 100., 550. + 100.),
//        };
//        let galley = self
//            .fonts
//            .layout_single_line(TextStyle::Small, ".hello world".to_string());
//        tessellator.tessellate_shape(
//            [1, 1],
//            Shape::Rect {
//                rect: rect2.clone(),
//                corner_radius: 0.0,
//                fill: Color32::BLACK,
//                stroke: Stroke::new(1., Color32::WHITE),
//            },
//            &mut mesh,
//        );
//        tessellator.tessellate_shape(
//            [1, 1],
//            Shape::Rect {
//                rect: rect.clone(),
//                corner_radius: 0.0,
//                fill: Color32::RED,
//                stroke: Stroke::new(25., Color32::WHITE),
//            },
//            &mut mesh,
//        );
//        let texture = self.fonts.texture();
//        let text = "hello world".to_string();
//        tessellator.tessellate_text(
//            [texture.width, texture.height],
//            Pos2::new(500., 500.),
//            &*galley,
//            Color32::WHITE,
//            false,
//            &mut mesh,
//        );
//        tessellator.tessellate_text(
//            [texture.width, texture.height],
//            Pos2::new(510., 510.),
//            &*galley,
//            Color32::WHITE,
//            false,
//            &mut mesh,
//        );
//        let mut paint_jobs = Vec::new();
//        //        dbg!(&mesh);
//        let clipped_mesh = ClippedMesh(
//            Rect {
//                min: Pos2::ZERO,
//                max: Pos2::new(self.size.width as f32, self.size.height as f32),
//            },
//            mesh,
//        );
//        paint_jobs.push(clipped_mesh);
//
//        let output = self.surface.get_current_frame()?.output;
//        let view = output
//            .texture
//            .create_view(&wgpu::TextureViewDescriptor::default());
//        let mut encoder = self
//            .device
//            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                label: Some("encoder"),
//            });
//        let screen_descriptor = ScreenDescriptor {
//            physical_width: self.size.width,
//            physical_height: self.size.height,
//            scale_factor: 1.,
//        };
//        //        let texture = epaint::Texture {
//        //            version: 0,
//        //            width: 1,
//        //            height: 1,
//        //            pixels: vec![255],
//        //        };
//        self.egui_rpass
//            .update_texture(&self.device, &self.queue, &texture);
//        self.egui_rpass
//            .update_user_textures(&self.device, &self.queue);
//        self.egui_rpass
//            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);
//
//        // Record all render passes.
//        self.egui_rpass
//            .execute(
//                &mut encoder,
//                &view,
//                //                &[],
//                &paint_jobs,
//                &screen_descriptor,
//                Some(wgpu::Color::BLUE),
//            )
//            .unwrap();
//        self.queue.submit(iter::once(encoder.finish()));
//        Ok(())
//    }
//    async fn new(window: &Window) -> Result<Self> {
//        let size = window.inner_size();
//        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
//        let surface = unsafe { instance.create_surface(window) };
//        println!("surface");
//        let adapter = instance
//            .request_adapter(&wgpu::RequestAdapterOptions {
//                power_preference: wgpu::PowerPreference::default(),
//                compatible_surface: Some(&surface),
//            })
//            .await
//            .unwrap();
//
//        println!("{:#?}", adapter.get_info());
//        println!("{:#?}", adapter.is_surface_supported(&surface));
//        let (device, queue) = adapter
//            .request_device(
//                &wgpu::DeviceDescriptor {
//                    features: wgpu::Features::empty(),
//                    limits: wgpu::Limits::default(),
//                    label: None,
//                },
//                None, // Trace path
//            )
//            .await
//            .unwrap();
//        let format = surface.get_preferred_format(&adapter).unwrap();
//
//        let config = wgpu::SurfaceConfiguration {
//            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//            //            format: TextureFormat::Rgba8UnormSrgb,
//            //            format: TextureFormat::Bgra8UnormSrgb,
//            format,
//            //            [Rgba8UnormSrgb, Bgra8UnormSrgb]
//            width: size.width,
//            height: size.height,
//            present_mode: wgpu::PresentMode::Fifo,
//        };
//        surface.configure(&device, &config);
//        let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, format, 1);
//        let fonts = Fonts::from_definitions(1., FontDefinitions::default());
//
//        Ok(Self {
//            instance,
//            adapter,
//            surface,
//            device,
//            queue,
//            config,
//            egui_rpass,
//            size,
//            fonts,
//            pos: 0.0,
//        })
//    }
//}
//
//fn main1() -> Result<()> {
//    env_logger::init();
//    //    bug2::main()?;
//    //    return Ok(());
//
//    let event_loop = EventLoop::new();
//    let window = WindowBuilder::new().build(&event_loop).unwrap();
//    // State::new uses async code, so we're going to wait for it to finish
//    //    let mut compute = pollster::block_on(compute::Compute::new(&window))?;
//    //    pollster::block_on(compute.compute())?;
//    //    return Ok(());
//    let mut state = pollster::block_on(EguiState::new(&window))?;
//    let mut left_hold = false;
//    let mut mouse_start_pos = Vec2::new(0., 0.);
//    let mut mouse_pos = Vec2::new(0., 0.);
//
//    event_loop.run(move |event, _, control_flow| {
//        match event {
//            Event::WindowEvent {
//                ref event,
//                window_id,
//            } if window_id == window.id() => {
//                if !state.input(event) {
//                    match event {
//                        WindowEvent::MouseInput {
//                            state: ElementState::Pressed,
//                            button: MouseButton::Left,
//                            ..
//                        } => {
//                            left_hold = true;
//                        }
//                        WindowEvent::MouseInput {
//                            state: ElementState::Released,
//                            button: MouseButton::Left,
//                            ..
//                        } => {
//                            left_hold = false;
//                        }
//                        WindowEvent::MouseWheel {
//                            delta: MouseScrollDelta::LineDelta(x, y),
//                            ..
//                        } => {
//                            // delta is a vector of [0., +-1.]
//                            state.zoom(*y);
//                        }
//                        WindowEvent::CursorMoved {
//                            device_id,
//                            position,
//                            modifiers,
//                        } => {
//                            let pos = Vec2::new(position.x as f32, position.y as f32);
//                            let delta = pos - mouse_pos;
//                            mouse_pos = pos;
//                            state.mouse_moved(mouse_pos);
//                            if left_hold {
//                                state.pan(delta);
//                            }
//                        }
//                        WindowEvent::CloseRequested
//                        | WindowEvent::KeyboardInput {
//                            input:
//                                KeyboardInput {
//                                    state: ElementState::Pressed,
//                                    virtual_keycode: Some(VirtualKeyCode::Escape),
//                                    ..
//                                },
//                            ..
//                        } => *control_flow = ControlFlow::Exit,
//                        WindowEvent::Resized(physical_size) => {
//                            state.resize(*physical_size);
//                        }
//                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                            state.resize(**new_inner_size);
//                        }
//                        _ => {}
//                    }
//                }
//            }
//            Event::RedrawRequested(_) => {
//                state.update();
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
//            }
//            Event::MainEventsCleared => {
//                // RedrawRequested will only trigger once, unless we manually
//                // request it.
//                window.request_redraw();
//            }
//            _ => {}
//        }
//    });
//}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "egui template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self { label, value } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            //            ui.add(scale::Scale {});
            //            ui.add(scale::ScaleEntry {
            //                text: "hello".into(),
            //            });
            ui.add(scale::Scale {});
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("egui template");
            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

use std::iter;
use std::time::Instant;

use chrono::{Timelike, Utc};
use egui::FontDefinitions;
use egui_wgpu_backend::{epi, RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use winit::dpi::PhysicalSize;
use winit::event::Event::*;
use winit::event::WindowEvent;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

/// A custom event type for the winit app.
#[derive(Debug)]
enum MyEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<MyEvent>>);

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(MyEvent::RequestRedraw)
            .ok();
    }
}

/// A simple egui + wgpu + winit based example.
fn main() {
    let event_loop = winit::event_loop::EventLoop::with_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("egui-wgpu_winit example")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::GL);
    let surface = unsafe { instance.create_surface(&window) };

    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (mut device, mut queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
        event_loop.create_proxy(),
    )));

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    // Display the demo application that ships with egui.
    let mut demo_app = TemplateApp::default();

    let start_time = Instant::now();
    let mut previous_frame_time = None;
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);
        match event {
            Event::WindowEvent {
                window_id: _window_id,
                event,
            } => match event {
                WindowEvent::Resized(..) => {
                    window.request_redraw();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    window.request_redraw();
                }
                WindowEvent::MouseInput {..} => {
                    window.request_redraw();
                }
                WindowEvent::MouseWheel {..} => {
                    window.request_redraw();
                }
                WindowEvent::CursorMoved {..} => {
                    window.request_redraw();
                }
                WindowEvent::CursorLeft { .. } => {
                    window.request_redraw();
                }
                WindowEvent::ModifiersChanged(..) => {
                    window.request_redraw();
                }
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => {
                *control_flow = ControlFlow::Exit;
                    window.request_redraw();
                }
                WindowEvent::ReceivedCharacter(..) => {
                    window.request_redraw();
                }
                _ => {}
            }
            Event::RedrawRequested(..) => {
                //                println!("redraw");
                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame.
                let egui_start = Instant::now();
                platform.begin_frame();
                let mut app_output = epi::backend::AppOutput::default();

                let mut frame = epi::backend::FrameBuilder {
                    info: epi::IntegrationInfo {
                        web_info: None,
                        cpu_usage: previous_frame_time,
                        seconds_since_midnight: Some(seconds_since_midnight()),
                        native_pixels_per_point: Some(window.scale_factor() as _),
                        prefer_dark_mode: None,
                    },
                    tex_allocator: &mut egui_rpass,
                    output: &mut app_output,
                    repaint_signal: repaint_signal.clone(),
                }
                .build();

                // Draw the demo application.
                demo_app.update(&platform.context(), &mut frame);

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let (_output, paint_commands) = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(paint_commands);

                let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
                println!("{} FPS", 1. / frame_time);
                previous_frame_time = Some(frame_time);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };
                egui_rpass.update_texture(&device, &queue, &platform.context().texture());
                egui_rpass.update_user_textures(&device, &queue);
                egui_rpass.update_buffers(&mut device, &mut queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();
                // Submit the commands.
                queue.submit(iter::once(encoder.finish()));

                //                // Redraw egui
                //                output_frame.present();

                // Suppport reactive on windows only, but not on linux.
                //                if _output.needs_repaint {
                //                    *control_flow = ControlFlow::Poll;
                //                } else {
                //                    *control_flow = ControlFlow::Wait;
                //                }
                //                *control_flow = ControlFlow::Poll;
                *control_flow = ControlFlow::Wait;
            }
            Event::MainEventsCleared
//            | UserEvent(Event::RequestRedraw)
            => {
//                println!("request redraw");
//                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}

/// Time of day as seconds since midnight. Used for clock in demo app.
pub fn seconds_since_midnight() -> f64 {
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}
