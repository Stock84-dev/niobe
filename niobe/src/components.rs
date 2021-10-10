use crate::combo::ChartCombo;
use crate::Layout;
use epaint::emath::{Pos2, Rect};
use epaint::{emath, Color32, Stroke, Tessellator};
use futures::StreamExt;
use nalgebra_glm::{TVec2, U32Vec2, Vec2};
use niobe_core::pipelines::line::{LineDrawer, LineStripPipeline};
use niobe_core::pipelines::mesh::MeshDrawer;
use niobe_core::pipelines::Drawer;
use niobe_core::UiPipeline;
use std::cell::RefCell;
use std::sync::Mutex;
use stretch::node::Node;
use stretch::style::{Dimension, Style};
use stretch::Stretch;
use wgpu::util::RenderEncoder;
use wgpu::{Device, Queue, RenderPass};
use wgpu_glyph::GlyphBrush;

pub mod scale;
pub mod scale_highlight;
pub mod view;

pub struct ChartState {
    node: Node,
    flexbox: Stretch,
    queue: Queue,
    pixel_scale: Vec2,
    mouse_pos: Vec2,
    scale: Vec2,
    translate: Vec2,
    plot_size: TVec2<u32>,
    position: Vec2,
}

impl ChartState {
    /// converts form top left (-1,1), bottom right (1,-1) to
    /// top left (0, 0) bottom right (pixel window width, pixel window height)
    pub fn screen_to_pixel_space(&self, screen_space: Vec2) -> U32Vec2 {
        U32Vec2::new(
            (screen_space.x + 1.) / self.pixel_scale.x as u32,
            (2. - screen_space.y) / self.pixel_scale.y as u32,
        )
    }

    pub fn pixel_to_screen_space(&self, pixel_space: U32Vec2) -> Vec2 {
        Vec2::new(pixel_space.x as f32, pixel_space.y as f32) * self.pixel_scale - 1.
    }
}

pub trait Component {
    fn on_mouse_moved(&mut self, state: &ChartCombo);
    fn on_zoom(&mut self, state: &ChartCombo);
    fn on_pan(&mut self, state: &ChartCombo);
    fn draw_ui(&mut self, combo: &ChartCombo, drawer: &mut UiPipeline);
    fn pipeline_kind(&self) -> PipelineKind;
    fn draw<'a>(&self, drawer: &mut RenderPass<'a>);
}

//impl<T: Component> Component for RefCell<T> {
//    fn layout_invalidated(&mut self, state: &ChartState) {
//        self.layout_invalidated()
//    }
//
//    fn on_mouse_moved(&mut self, state: &ChartState) {
//        self.on_mouse_moved()
//    }
//
//    fn on_zoom(&mut self, state: &ChartState) {
//        self.on_zoom()
//    }
//
//    fn on_pan(&mut self, state: &ChartState) {
//        self.on_pan()
//    }
//
//    fn draw(&mut self, drawer: &mut UiPipeline) {
//        self.draw()
//    }
//}
//
pub struct ComponentBase {
    pub node: Node,
    pub fill_color: Color32,
    pub border_color: Color32,
}

impl ComponentBase {
    pub fn draw(&self, combo: &ChartCombo, drawer: &mut UiPipeline) {
        let style = combo.flexbox.style(self.node).unwrap();
        let rect = combo.component_rect(self.node);
        match style.border.start {
            Dimension::Points(width) => {
                let width = width / 2.;
                let start = Pos2::new(rect.min.x - width, rect.min.y - width);
                let end = Pos2::new(rect.min.x - width, rect.max.y + width);
                drawer.line(start, end, Stroke::new(width, self.border_color));
            }
            _ => {}
        }
        match style.border.end {
            Dimension::Points(width) => {
                let width = width / 2.;
                let start = Pos2::new(rect.max.x + width, rect.min.y - width);
                let end = Pos2::new(rect.max.x + width, rect.max.y + width);
                drawer.line(start, end, Stroke::new(width, self.border_color));
            }
            _ => {}
        }
        match style.border.top {
            Dimension::Points(width) => {
                let width = width / 2.;
                let start = Pos2::new(rect.min.x - width, rect.min.y - width);
                let end = Pos2::new(rect.max.x + width, rect.min.y - width);
                drawer.line(start, end, Stroke::new(width, self.border_color));
            }
            _ => {}
        }
        match style.border.bottom {
            Dimension::Points(width) => {
                let width = width / 2.;
                let start = Pos2::new(rect.min.x - width, rect.max.y + width);
                let end = Pos2::new(rect.max.x + width, rect.max.y + width);
                drawer.line(start, end, Stroke::new(width, self.border_color));
            }
            _ => {}
        }
        drawer.rect(rect, 0., self.fill_color, Stroke::default());
    }

    pub fn set_scissor_rect(&self, combo: &ChartCombo, render_pass: &mut RenderPass) {
        let rect = combo.component_rect(self.node);
        render_pass.set_scissor_rect(
            rect.min.x as u32,
            rect.min.y as u32,
            rect.max.x as u32,
            rect.max.y as u32,
        );
    }
}

#[derive(Clone, Copy)]
pub enum PipelineKind {
    Ui,
    LineStrip,
    Custom(usize),
}
