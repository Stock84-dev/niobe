use crate::combo::ChartCombo;
use crate::series::{DrawControlFlow, Series};
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::line::{LineBindGroup, LineDrawer, LineStripPipeline, LineUniform};
use niobe_core::pipelines::Drawer;
use niobe_core::Point2d;
use wgpu::util::RenderEncoder;

struct LineSeries {
    line_ubd: [LineUniform; 1],
    line_ubo: Buffer<LineUniform>,
    line_ebo: Buffer<Point2d>,
    line_bind_group: LineBindGroup,
}
