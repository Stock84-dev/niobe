use crate::combo::ChartCombo;
use crate::components::scale::Orientation;
use crate::components::{Component, PipelineKind};
use epaint::emath::Rect;
use nalgebra_glm::Vec2;
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{LineDrawer, LineStripPipeline};
use niobe_core::pipelines::ui::UiRenderPass;
use niobe_core::pipelines::Drawer;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::RenderEncoder;
use wgpu::{BindGroup, IndexFormat, RenderPass};

pub mod line;

#[repr(u8)]
pub enum DrawControlFlow {
    Finished = 0,
    DrawRequested = 1,
}

pub struct Series {
    instance_range: Range<u32>,
    vertex_range: Range<u32>,
    bind_group: Arc<BindGroup>,
    buffers: HashMap<u32, Arc<Buffer>>,
    index: Option<(Arc<Buffer>, IndexFormat)>,
    pipeline_kind: PipelineKind,
}

impl Series {
    pub fn set_instance_range(&mut self, range: Range<u32>) {
        self.instance_range = range;
    }

    pub fn set_bind_group(&mut self, bind_group: Arc<BindGroup>) {
        self.bind_group = bind_group;
    }

    pub fn set_buffer(&mut self, id: u32, buffer: Arc<Buffer>) {
        self.buffers.insert(id, buffer);
    }

    pub fn set_index_buffer(&mut self, buffer: Arc<Buffer>, format: IndexFormat) {
        self.index = Some((buffer, format));
    }
}

impl Component for Series {
    fn on_mouse_moved(&mut self, state: &ChartCombo) {}

    fn on_zoom(&mut self, state: &ChartCombo) {}

    fn on_pan(&mut self, state: &ChartCombo) {}

    fn draw_ui(&mut self, combo: &ChartCombo, drawer: &mut UiRenderPass) {}

    fn pipeline_kind(&self) -> PipelineKind {
        self.pipeline_kind
    }

    fn draw<'a>(&self, drawer: &mut RenderPass<'a>) {
        drawer.set_bind_group(0, &self.bind_group, &[0]);
        for (id, buffer) in &self.buffers {
            drawer.set_vertex_buffer(*id, buffer.slice(..));
        }
        if let Some((buffer, format)) = &self.index {
            drawer.set_index_buffer(buffer.slice(..), *format);
            drawer.draw_indexed(0..buffer.len, 0, self.instance_range);
        } else {
            drawer.draw(self.vertex_range, self.instance_range);
        }
    }
}

mod exp {
    use crate::components::scale::Orientation;
    use epaint::emath::Rect;
    use std::sync::Arc;
    use wgpu::BufferSlice;

    // every event that happens in view bundle gets broadcasted to inner views
    // then each view can calculate if mouse is over itself and apply translation
    // each view can respond to different events eg. scale in x direction or y, translate in x or ...
    // each component should be able to emit events to bundle,
    // eg. left mouse hold and move over scale -> scale event
    // view.set_position() -> translate event
    pub struct ViewBundle {}

    pub struct View {
        rect: Rect,
        series: Vec<Series>,
        bundle: Arc<ViewBundle>,
    }

    pub struct Scale {
        attached_view: Arc<View>,
        orientation: Orientation,
        bundle: Arc<ViewBundle>,
        highlights: Vec<Highlight>,
    }

    // allow to show value on a scale with custom position and text
    pub struct Highlight {
        // color
    // text
    // pos
    }

    pub struct Crosshair {
        scales: Arc<Scale>,
        bundle: Arc<ViewBundle>,
        // if mouse position is in scale.attached_view.rect then set scale highlight
    }

    pub struct Grid {}

    pub struct Series {}

    enum DrawCommand<'a> {
        Line {
            x_buffer: BufferSlice<'a>,
            y_buffer: BufferSlice<'a>,
        },
    }
    ring buffer

view
-hashmap<i8, Vec<series>>

pipelines

series
- uniform Arc<BufferSlice>
- buffer slices
- index buffer?

line pipeline
draw from ring buffer





components:
grid
series
scale
crosshair (optional: move by closest point)
legend
legend with hover value
end value on scale
view(pos, size)

instead of event approach we go with immediate but retained approach

trait Drawable {
	fn draw(&self, commands: &mut HashMap<i8, HashMap<PipelineKind, Vec<Command>>>);
}

enum Command {
	Bundle(RenderBundle),
	SetScissorRect,
	SetPrevScissorRect


}

impl Series {
	set_pipeline();
	set_z()l;
	render_bundle() -> &mut rendrbundle;
}

}
