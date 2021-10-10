use crate::components::{Component, ChartState, ComponentBase};
use epaint::{Tessellator, Mesh, Shape, Stroke, Color32, emath, TextStyle};
use crate::{Layout, Layoutable};
use glyph_brush::{OwnedSection, OwnedText};
use nalgebra_glm::{TVec2, Vec2};
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{LineBindGroup, LineDrawer, LineStripPipeline, LineUniform};
use niobe_core::pipelines::mesh::MeshDrawer;
use niobe_core::{Point2d, UiPipeline, ComponentColors};
use rgb::RGBA;
use std::num::NonZeroUsize;
use wgpu::util::RenderEncoder;
use wgpu::{BufferUsages, Device};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::{GlyphBrush, HorizontalAlign, VerticalAlign};
use stretch::node::Node;
use stretch::style::{Style, Dimension};
use epaint::emath::{Rect, Pos2};

#[derive(Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct ScaleConfig {
    pub orientation: Orientation,
    pub max_ticks: u32,
    pub border_color: RGBA<f32>,
    pub border_width: u32,
    pub size: u32,
    pub font_size: Vec2,
    pub font_color: RGBA<f32>,
    pub font: FontArc,
    pub crosshair_background_color: Option<RGBA<f32>>,
}

impl Default for ScaleConfig {
    fn default() -> Self {
        Self {
            max_ticks: 50,
            size: 50,
            font: FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf")).unwrap(),
        }
    }
}

pub struct Scale {
    base: ComponentBase,
    node: Node,
    colors: ComponentColors,
    text_color: Color32,
    text_style: TextStyle,
    sections: Vec<OwnedSection>,
    orientation: Orientation,
    write_options: lexical::write_float_options::Options,
}

impl Scale {
    fn draw(&mut self, state: &ChartState, drawer: &mut UiPipeline) {
        let rect = 
        match self.orientation {
            Orientation::Horizontal => {
                let mut i = 0;
                every_log10_x(state, |pos| {
                    let text = lexical::to_string_with_options::<_, FORMAT>(pos.value, &write_options);
                    drawer.text_single_line(self.text_style, text, pos., self.text_color)
                    self.x_sections[i].screen_position = (
                        (pos.screen_pos + 1.) / state.pixel_scale.x,

                        1.95 / state.pixel_scale.y,
                    );
                    i += 1;
                });
                for section in self.sections.iter_mut().skip(i) {
                    section.text[0].text.clear();
                }

            }
            Orientation::Vertical => {

            }
        }
    }

    pub fn new(config: &ScaleConfig, device: &Device, this: Node) -> (Self, i8) {
        let border_ubd = [LineUniform {
            color: config.border_color,
            scale: Vec2::new(1., 1.),
            translate: Vec2::new(0.0, 0.0),
            line_scale: Vec2::new(0.01, 0.01),
        }];
        let border_ubo = Buffer::new(
            &device,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &border_ubd,
        );
        let border_vbd;
        let section_builder = section_builder(config.orientation);
        match config.orientation {
            Orientation::Horizontal => {
                border_vbd = [Point2d::new(-1., 0.), Point2d::new(1., 0.)];
            }
            Orientation::Vertical => {
                border_vbd = [Point2d::new(0., -1.), Point2d::new(0., 1.)];
            }
        };
        let sections = (0..config.max_ticks)
            .into_iter()
            .map(section_builder)
            .collect();
        let border_vbo = Buffer::new(&device, BufferUsages::VERTEX, &border_vbd);
        let border_group = LineBindGroup::new(&device, &border_ubo.slice(..));
        (
            Self {
                layout: match config.orientation {
                    Orientation::Horizontal => {
                        Layout { position: Vec2::new(0., ), size: Default::default() },

                    }
                    Orientation::Vertical => {
                        Layout { position: k, size: Default::default() },
                    }
                },
                sections,
                orientation: config.orientation,
                border_width: config.border_width,
                border_ubd,
                border_vbd,
                border_vbo,
                border_ubo,
                border_group,
                write_options: lexical::WriteFloatOptions::builder()
                    .max_significant_digits(NonZeroUsize::new(2))
                    .round_mode(RoundMode::Round)
                    .trim_floats(true)
                    .decimal_point(b'.')
                    .build()
                    .unwrap(),
            },
            0,
        )
    }

    pub fn get_value_at(&self, state: &ChartState, pos: f32) -> f32 {
        match self.orientation {
            Orientation::Horizontal => {
                (-1. - state.translate.x + state.position.x * state.pixel_scale.x) / state.scale.x
            }
            Orientation::Vertical => {
                (-1. - state.translate.y + state.position.y * state.pixel_scale.y) / state.scale.y
            }
        }
    }

    pub fn convert_to_text(&self, value: f32) -> String {
        lexical::to_string_with_options::<_, FORMAT>(value, &self.write_options)
    }

    pub fn section_builder<T>(&self) -> impl FnMut(T) -> OwnedSection {
        section_builder(self.orientation)
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    fn update_x_ticks(
        &mut self,
        state: &ChartState,
        write_options: &lexical::write_float_options::Options,
    ) {
        let mut i = 0;
        every_log10_x(state, |pos| {
            self.x_sections[i].screen_position = (
                (pos.screen_pos + 1.) / state.pixel_scale.x,
                1.95 / state.pixel_scale.y,
            );
            self.x_sections[i].text[0].text =
                lexical::to_string_with_options::<_, FORMAT>(pos.value, &write_options);
            i += 1;
        });
        for section in self.sections.iter_mut().skip(i) {
            section.text[0].text.clear();
        }
    }

    fn update_y_ticks(
        &mut self,
        state: &ChartState,
        write_options: &lexical::write_float_options::Options,
    ) {
        let mut i = 0;
        every_log10_y(state, |pos| {
            self.y_sections[i].screen_position = (
                (pos.screen_pos + 1.) / state.piyel_scale.y,
                1.95 / state.piyel_scale.y,
            );
            self.y_sections[i].teyt[0].teyt =
                lexical::to_string_with_options::<_, FORMAT>(pos.value, &write_options);
            i += 1;
        });
        for section in self.sections.iter_mut().skip(i) {
            section.text[0].text.clear();
        }
    }
}

impl Component for Scale {
    fn layout_invalidated(&mut self, state: &ChartState) {
        self.border_ubd[0].line_scale.x = self.border_width as f32 * state.pixel_scale.x;
        self.border_ubd[0].line_scale.y = self.border_width as f32 * state.pixel_scale.y;
    }

    fn on_zoom(&mut self, state: &ChartState) {
        match self.orientation {
            Orientation::Horizontal => self.update_x_ticks(state, &self.write_options),
            Orientation::Vertical => self.update_y_ticks(state, &self.write_options),
        }
    }

    fn on_pan(&mut self, state: &ChartState) {
        match self.orientation {
            Orientation::Horizontal => self.update_x_ticks(state, &self.write_options),
            Orientation::Vertical => self.update_y_ticks(state, &self.write_options),
        }
    }

    fn draw_line_strip<'s, 'e>(
        &'s mut self,
        drawer: &mut LineDrawer<'s, 'e, &dyn RenderEncoder<'s>, LineStripPipeline>,
    ) {
        drawer
            .set_bind_group(&self.border_group, 0)
            .draw(self.border_vbd.slice(..));
    }

    fn draw_text(&mut self, drawer: &mut GlyphBrush<()>) {

        for section in &self.sections {
            drawer.queue(section);
        }
    }
}

impl Layoutable for Scale {
    fn layout(&self, state: &ChartState) -> &Layout {
        state.flexbox.layout(self.node).unwrap().into()
    }
}

pub struct Log10Pos {
    screen_pos: f32,
    value: f32,
}

pub fn every_log10_x(state: &ChartState, callback: impl FnMut(Log10Pos)) {
    let mut log = 10.0f32.powf(-state.scale.x.log10().floor()) / 100.;
    let start = (-1. - state.translate.x + state.position.x * state.pixel_scale.x) / state.scale.x;
    let mut x_value = if start > 0. {
        start - start.rem(log) + log
    } else {
        start - start.rem(log)
    };
    let mut x = x_value * state.scale.x + state.translate.x;
    let mut tick_spacing = log * state.scale.x;
    if 2. / tick_spacing > 50. {
        log *= 10.;
        tick_spacing *= 10.;
        x_value = start - start.rem(log);
        x = x_value * state.scale.x + state.translate.x;
    }

    while x < 1. {
        callback(Log10Pos {
            screen_pos: x,
            value: x_value,
        });

        x += tick_spacing;
        x_value += log;
    }
}

pub fn every_log10_y(state: &ChartState, callback: impl FnMut(Log10Pos)) {
    let mut log = 10.0f32.powf(-state.scale.y.log10().floor()) / 100.;
    let start = (-1. - state.translate.y + state.position.y * state.pixel_scale.y) / state.scale.y;
    let mut y_value = if start > 0. {
        start - start.rem(log) + log
    } else {
        start - start.rem(log)
    };
    let mut y = y_value * state.scale.y + state.translate.y;
    let mut tick_spacing = log * state.scale.y;
    if 2. / tick_spacing > 50. {
        log *= 10.;
        tick_spacing *= 10.;
        y_value = start - start.rem(log);
        y = y_value * state.scale.y + state.translate.y;
    }

    let n_y_ticks = 2. / tick_spacing;
    while y < 1. {
        callback(Log10Pos {
            screen_pos: y,
            value: y_value,
        });

        y += tick_spacing;
        y_value += log;
    }
}

fn section_builder<T>(orientation: Orientation) -> impl FnMut(T) -> OwnedSection {
    match orientation {
        Orientation::Horizontal => |_| OwnedSection {
            layout: Layout::default_single_line().h_align(HorizontalAlign::Center),
            text: vec![OwnedText::new("".to_string())
                .with_scale(config.font_size)
                .with_color(config.font_color)],
            ..Default::default()
        },
        Orientation::Vertical => |_| OwnedSection {
            layout: Layout::default_single_line().v_align(VerticalAlign::Center),
            text: vec![OwnedText::new("".to_string())
                .with_scale(config.font_size)
                .with_color(config.font_color)],
            ..Default::default()
        },
    }
}
