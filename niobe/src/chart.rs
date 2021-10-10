use crate::components::ChartState;
use futures::executor::{LocalPool, LocalSpawner};
use futures::StreamExt;
use glyph_brush::ab_glyph::PxScale;
use glyph_brush::Layout;
use glyph_brush::{OwnedSection, OwnedText};
use lexical::write_float_options::RoundMode;
use nalgebra_glm::{TVec2, Vec2};
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{LineBindGroup, LineDrawer, LineStripPipeline, LineUniform};
use niobe_core::pipelines::mesh::{MeshBindGroup, MeshDrawer, MeshUniform};
use niobe_core::Point2d;
use rgb::RGBA;
use std::num::NonZeroUsize;
use tap::{Tap, TapOptional};
use wgpu::util::{RenderEncoder, StagingBelt};
use wgpu::{BufferUsages, Device, Queue, TextureFormat};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Section, VerticalAlign};

pub struct Chart {}

pub struct ChartConfig<'a> {
    bg_color: RGBA<f32>,
    crosshair: Option<&'a CrosshairConfig>,
    scale: Option<&'a ScaleConfig>,
    grid: Option<&'a GridConfig>,
}

impl Chart {
    pub fn new(&self, device: &Device, config: &ChartConfig) -> Chart {
        Chart {}
    }

    pub fn draw(&mut self) {}

    pub fn resize(&mut self) {}

    pub fn zoom(&mut self) {}

    pub fn pan(&mut self) {}

    pub fn cursor_moved(&mut self) {}
}

struct Crosshair {
    sections: Vec<OwnedSection>,
}

impl Crosshair {
    fn new(config: &CrosshairConfig, scale_config: &Option<&ScaleConfig>) -> Self {
        let mut sections = Vec::new();
        if let Some(scale_config) = scale_config {
            sections = vec![
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
        }
        Self {}
    }
}

struct CrosshairScale {}

impl CrosshairScale {
    fn new(background_color: RGBA<f32>, device: &Device) -> Self {
        Self {
            mesh_ubo,
            mesh_uniforms,
            mesh_group,
            mesh_instance_vbo,
            mesh_instance_positions,
        }
    }

    fn on_mouse_moved(
        &mut self,
        state: &ChartState,
        write_options: &lexical::write_float_options::Options,
    ) {
    }

    fn on_zoom(
        &mut self,
        state: &ChartState,
        write_options: &lexical::write_float_options::Options,
    ) {
    }

    fn draw_mesh<'s, 'e, E: RenderEncoder<'s>>(
        &'s mut self,
        drawer: &mut MeshDrawer<'e, E>,
        pass_id: u8,
    ) {
        self.crosshair_scale
            .tap_mut(|x| x.draw_mesh(drawer, pass_id));
    }

    fn draw_line_strip<'s, 'e, E: RenderEncoder<'s>>(
        &'s mut self,
        drawer: &mut LineDrawer<'s, 'e, E, LineStripPipeline>,
        pass_id: u8,
    ) {
        drawer
            .bind_group(&self.line_group, 0)
            .draw(self.border_vbo.slice(..));
    }

    fn draw_text(&mut self, drawer: &mut GlyphBrush<()>, pass_id: u8) {
        self.x_scale.tap_mut(|x| x.draw_text(drawer, pass_id));
        self.crosshair_scale
            .tap_mut(|x| x.draw_text(drawer, pass_id));
    }
}

struct Scales {
    x_scale: Option<Scale>,
    y_scale: Option<Scale>,
    glyph_brush: GlyphBrush<()>,
    size: TVec2<u32>,
    staging_belt: StagingBelt,
    pool: LocalPool,
    spawner: LocalSpawner,
    border_vbo: Buffer<Point2d>,
    border_ubo: Buffer<LineUniform>,
    border_width: u32,
    border_uniform: [LineUniform; 1],
    write_options: lexical::write_float_options::Options,
    crosshair_scale: Option<CrosshairScale>,
    line_group: LineBindGroup,
}

impl Scales {
    fn new(config: &ScaleConfig, device: &Device, texture_format: TextureFormat) -> Self {
        let mut x_scale = None;
        let mut y_scale = None;
        if config.max_x_ticks > 0 {
            x_scale = Some(Scale {
                sections: (0..config.max_x_ticks)
                    .into_iter()
                    .map(build_x_section)
                    .collect(),
                crosshair_section: build_x_section(()),
            });
        }
        if config.max_y_ticks > 0 {
            y_scale = Some(Scale {
                sections: (0..config.max_y_ticks)
                    .into_iter()
                    .map(build_y_section)
                    .collect(),
                crosshair_section: build_y_section(()),
            });
        }
        let mut glyph_brush =
            GlyphBrushBuilder::using_font(config.font).build(device, texture_format);
        let borders = vec![
            Point2d::new(-1.0f32, -1.0),
            Point2d::new(1.0, -1.0),
            Point2d::new(1.0, 1.0),
            Point2d::new(-1.0, 1.0),
            Point2d::new(-1.0, -1.0),
        ];
        let pool = futures::executor::LocalPool::new();
        Self {
            x_scale,
            y_scale,
            glyph_brush,
            size: config.size,
            staging_belt: wgpu::util::StagingBelt::new(1024),
            spawner: pool.spawner(),
            pool,
            border_vbo: Buffer::new(&device, BufferUsages::VERTEX, &borders),
            line_group: LineBindGroup::new(&device, &border_ubo.slice(..)),
            border_ubo,
            border_width: config.border_width,
            border_uniform,
            write_options: lexical::WriteFloatOptions::builder()
                .max_significant_digits(NonZeroUsize::new(2))
                .round_mode(RoundMode::Round)
                .trim_floats(true)
                .decimal_point(b'.')
                .build()
                .unwrap(),
            crosshair_scale: if let Some(color) = config.crosshair_background_color {
                Some(CrosshairScale::new(color, &device))
            } else {
                None
            },
        }
    }

    fn on_window_resized(&mut self, state: &ChartState) {
        let scale = Vec2::all(1.) - self.size.into() * 2 * state.pixel_scale;
        self.border_uniform[0].scale = scale;
        self.border_uniform[0].line_scale = state.pixel_scale * self.border_width as f32;
        self.border_ubo
            .write_sliced(&state.queue, .., &self.border_uniform);
    }

    fn on_mouse_moved(&mut self, state: &ChartState) {
        self.crosshair_scale
            .tap_some(|x| x.on_mouse_moved(&state, &self.write_options));
    }

    fn on_zoom(&mut self, state: &ChartState) {
        self.crosshair_scale
            .tap_some(|x| x.on_zoom(&state, &self.write_options));
        self.update_ticks(&state);
    }

    fn on_pan(&mut self, state: &ChartState) {
        self.update_ticks(&state);
    }

    fn draw_mesh<'s, 'e, E: RenderEncoder<'s>>(
        &'s mut self,
        drawer: &mut MeshDrawer<'e, E>,
        pass_id: u8,
    ) {
        self.crosshair_scale
            .tap_mut(|x| x.draw_mesh(drawer, pass_id));
    }

    fn draw_line_strip<'s, 'e, E: RenderEncoder<'s>>(
        &'s mut self,
        drawer: &mut LineDrawer<'s, 'e, E, LineStripPipeline>,
        pass_id: u8,
    ) {
        drawer
            .bind_group(&self.line_group, 0)
            .draw(self.border_vbo.slice(..));
    }

    fn draw_text(&mut self, drawer: &mut GlyphBrush<()>, pass_id: u8) {
        self.x_scale.tap_mut(|x| x.draw_text(drawer, pass_id));
        self.crosshair_scale
            .tap_mut(|x| x.draw_text(drawer, pass_id));
    }

    fn update_ticks(&mut self, state: &ChartState) {
        self.x_scale
            .tap_some_mut(|x| x.update_x_ticks(state, self.size.x, &self.write_options));
        self.y_scale
            .tap_some_mut(|x| x.update_y_ticks(state, self.size.y, &self.write_options));
    }
}

pub struct CrosshairConfig {
    pub line_color: RGBA<f32>,
    pub line_width: TVec2<u32>,
}

pub struct GridConfig {
    pub line_color: RGBA<f32>,
    pub line_width: TVec2<u32>,
}

const FORMAT: u128 = lexical::format::STANDARD;
