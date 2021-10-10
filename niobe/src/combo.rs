use crate::components::view::{ChartView, DrawControlFlow, View};
use crate::components::Component;
use epaint::emath::{Pos2, Rect, Vec2};
use niobe_core::pipelines::line::{LineDrawer, LineStripPipeline};
use niobe_core::pipelines::ui::UiDrawer;
use niobe_core::pipelines::Drawer;
use std::cell::RefCell;
use std::rc::Rc;
use stretch::node::Node;
use stretch::style::Style;
use stretch::Stretch;
use wgpu::util::RenderEncoder;
use wgpu::RenderPass;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

macro_rules! impl_draw {
    ($name:ident, $kind:ty) => {
        pub fn $name<'s, 'e>(&mut self, drawer: &mut Drawer<'e, $kind>) -> DrawControlFlow {
            let mut control_flow = 0u8;
            for view in &mut self.views {
                control_flow |= view.borrow_mut().$name(self, drawer) as u8;
            }
            unsafe { std::mem::transmute(control_flow) }
        }
    };
}

pub struct ChartCombo {
    flexbox: Stretch,
    pub node: Node,
    left_hold: bool,
    mouse_pixel_pos: Pos2,
    views: Vec<View>,
    components: Vec<Rc<dyn Component>>,
}

impl ChartCombo {
    pub fn new() -> Self {
        let mut flexbox = Stretch::new();
        let node = flexbox.new_node(Style::default(), vec![]).unwrap();

        ChartCombo {
            flexbox,
            node,
            left_hold: false,
            mouse_pixel_pos: Default::default(),
            views: vec![],
            components: vec![],
        }
    }

    pub fn component_rect(&self, node: Node) -> Rect {
        let layout = self.flexbox.layout(node).unwrap();
        Rect::from_min_size(
            Pos2::new(layout.location.x, layout.location.y),
            Vec2::new(layout.size.width, layout.size.height),
        )
    }

    impl_draw!(
        draw_line_strip,
        LineDrawer<'s, 'e, &dyn RenderEncoder, LineStripPipeline>
    );

    pub fn draw_ui(&mut self, drawer: &mut UiDrawer) {
        self.views.iter().for_each(|x| x.draw_ui(delta));
        self.components.iter().for_each(|x| x.draw_ui(delta));
    }

    pub fn zoom(&mut self, mut delta: f32) {
        let base = 0.05;
        if delta.is_sign_positive() {
            delta += base;
        } else {
            delta = 1. - delta.abs() * base;
        }
        self.views.iter().for_each(|x| x.on_zoom(self, delta));
        self.components.iter().for_each(|x| x.on_zoom(self, delta));
    }

    pub fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.left_hold = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.left_hold = false;
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                // delta is a vector of [0., +-1.]
                self.zoom(*y);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            } => {
                let pos = Pos2::new(position.x as f32, position.y as f32);
                let delta = pos - self.mouse_pixel_pos;
                self.mouse_pixel_pos = pos;
                self.views
                    .iter()
                    .for_each(|x| x.on_mouse_moved(self, delta));
                self.components.iter().for_each(|x| x.on_mouse_moved(self));
                if left_hold {
                    self.views.iter().for_each(|x| x.on_pan(self, delta));
                    self.components.iter().for_each(|x| x.on_pan(self, delta));
                }
            }
            _ => {}
        }
    }
}
