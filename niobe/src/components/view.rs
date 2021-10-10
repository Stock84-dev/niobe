use crate::combo::ChartCombo;
use crate::components::{ChartState, Component, ComponentBase};
use crate::series::DrawControlFlow;
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{LineBindGroup, LineDrawer, LineStripPipeline, LineUniform};
use niobe_core::pipelines::Drawer;
use niobe_core::Point2d;
use stretch::node::Node;
use wgpu::util::RenderEncoder;

pub struct View {
    base: ComponentBase,
    series: Vec<Box<dyn Series>>,
}

impl View {
    fn draw_line_strip<'s, 'e>(
        &'s mut self,
        combo: &ChartCombo,
        drawer: &mut Drawer<'e, LineDrawer<'s, 'e, &dyn RenderEncoder, LineStripPipeline>>,
    ) -> DrawControlFlow {
        self.base.set_scissor_rect(combo, drawer.pass);
        let mut control_flow = 0u8;
        for series in &mut self.series {
            control_flow |= series.draw_line_strip(combo, drawer) as u8;
        }
        unsafe { std::mem::transmute(control_flow) }
    }
}

impl Component for View {
    fn on_mouse_moved(&mut self, state: &ChartState) {}

    fn on_zoom(&mut self, state: &ChartState) {
        self.line_ubd[0].line_scale *= 2. - delta;
        self.line_ubd[0].scale *= delta
    }

    fn on_pan(&mut self, state: &ChartState) {
        self.line_ubd[0].translate += delta * 2.;
    }

    fn draw_ui(&mut self, combo: &ChartCombo, drawer: &mut UiPipeline) {
        self.base.draw(combo, drawer)
    }
}
