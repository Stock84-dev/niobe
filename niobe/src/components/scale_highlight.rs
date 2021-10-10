use crate::components::scale::{Orientation, Scale};
use crate::components::{Component, ChartState};
use crate::Layoutable;
use glyph_brush::OwnedSection;
use nalgebra_glm::Vec2;
use niobe_core::buffer::Buffer;
use niobe_core::pipelines::mesh::{MeshBindGroup, MeshDrawer, MeshUniform};
use niobe_core::Point2d;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu::util::RenderEncoder;
use wgpu::{BufferUsages, Device};
use wgpu_glyph::GlyphBrush;

pub enum ScaleHighlightKind {
    FollowMouse,
}

pub struct ScaleHighlightConfig {
    pub kind: ScaleHighlightKind,
}

pub struct ScaleHighlight {
    scale: Rc<RefCell<Scale>>,
    section: OwnedSection,
    mesh_group: MeshBindGroup,
    mesh_ebo: Buffer<Point2d>,
    mesh_ibo: Buffer<u16>,
    mesh_ubo: Buffer<MeshUniform>,
    mesh_ebd: Vec<Point2d>,
    mesh_ubd: Vec<MeshUniform>,
}

impl ScaleHighlight {
    pub fn new(
        config: &ScaleHighlightConfig,
        device: &Device,
        scale: Rc<RefCell<Scale>>,
    ) -> (Self, i8) {
        let mesh_ubd = vec![MeshUniform {
            color: background_color,
            scale: Vec2::new(1., 1.),
            translate: Vec2::new(0., 0.),
            mesh_scale: Vec2::new(1., 1.),
        }];
        let mesh_ubo = Buffer::new(
            &device,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &mesh_ubd,
        );
        let mesh_group = MeshBindGroup::new(&device, mesh_ubo.slice(..));
        let mesh_ebd = vec![Point2d::default(), Point2d::default()];
        let mesh_ebo = Buffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            &mesh_ebd,
        );
        let mesh_ibd = vec![0u16, 1, 2, 2, 3, 0];
        let mesh_ibo = Buffer::new(&device, BufferUsages::INDEX, &mesh_ibd);

        (
            Self {
                section: (scale.borrow().section_builder())(),
                scale,
                mesh_ubo,
                mesh_ubd,
                mesh_group,
                mesh_ebo,
                mesh_ebd,
                mesh_ibo,
            },
            1,
        )
    }

    fn update_highlight_x_values(&mut self, state: &ChartState) {
        let mut scale = self.scale.borrow();
        let value = scale.get_value_at(&state, state.mouse_pos.x);
        let pos = Vec2::new(state.mouse_pos.x, scale.layout().position.y);
        let pos = state.screen_to_pixel_space(pos);
        self.section.screen_position = (pos.x as f32, pos.y as f32);
        self.section.text[0].text = scale.convert_to_text(value);
    }

    fn update_highlight_y_values(&mut self, state: &ChartState) {
        let mut scale = self.scale.borrow();
        let value = scale.get_value_at(&state, state.mouse_pos.y);
        let pos = Vec2::new(scale.layout().position.x, state.mouse_pos.y);
        let pos = state.screen_to_pixel_space(pos);
        self.section.screen_position = (pos.x as f32, pos.y as f32);
        self.section.text[0].text = scale.convert_to_text(value);
    }
}

impl Component for ScaleHighlight {
    fn on_mouse_moved(&mut self, state: &ChartState) {
        self.mesh_ubd[0].translate = state.mouse_pos;
        let orientation = self.scale.borrow().orientation();
        match orientation {
            Orientation::Horizontal => self.update_highlight_x_values(state),
            Orientation::Vertical => self.update_highlight_y_values(state),
        }
        let orientation = orientation as usize;
        self.mesh_ebd[0][orientation] = state.mouse_pos[orientation];
        self.mesh_ebo.write_sliced(&self.queue, .., &self.mesh_ebd);
    }

    fn on_zoom(&mut self, state: &ChartState) {
        let orientation = self.scale.borrow().orientation();
        match orientation {
            Orientation::Horizontal => self.update_highlight_x_values(state),
            Orientation::Vertical => self.update_highlight_y_values(state),
        }
    }

    fn draw_mesh<'s, 'e>(&'s mut self, drawer: &mut MeshDrawer<'e, &dyn RenderEncoder<'s>>) {
        drawer
            .set_bind_group(&self.mesh_group, 0)
            .set_indices(self.mesh_ibo.slice(..))
            .set_vertices(self.mesh_vbo.slice(..))
            .draw(self.mesh_ebo.slice(..));
    }

    fn draw_text(&mut self, drawer: &mut GlyphBrush<()>) {
        drawer.queue(&self.section);
    }
}
