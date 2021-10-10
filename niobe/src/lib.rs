#[macro_use]
extern crate getset;
#[macro_use]
extern crate derive_builder;

pub mod chart;
pub mod combo;
pub mod components;
pub mod context;
pub mod series;
use nalgebra_glm::Vec2;

struct Layout {
    position: Vec2,
    size: Vec2,
}

impl From<stretch::result::Layout> for Layout {
    fn from(layout: stretch::Layout) -> Self {
        Layout {
            position: Vec2(layout.location.x, layout.location.y),
            size: Vec2::new(layout.size.width, layout.size.height),
        }
    }
}

trait Layoutable {
    fn layout(&self, state: &ChartState) -> &Layout;
}
