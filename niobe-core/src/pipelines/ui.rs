use egui_wgpu_backend::ScreenDescriptor;
use epaint::emath::{Pos2, Rect};
use epaint::text::{FontDefinitions, Fonts};
use epaint::{
    ClippedMesh, Color32, Mesh, Shape, Stroke, TessellationOptions, Tessellator, TextStyle, Texture,
};
use std::iter;
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

pub struct UiRenderPass {
    render_pass: egui_wgpu_backend::RenderPass,
    tessellator: Tessellator,
    fonts: Fonts,
    texture: Arc<Texture>,
    paint_jobs: Vec<ClippedMesh>,
}

impl UiRenderPass {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, format, 1);
        let tessellator = Tessellator::from_options(TessellationOptions::default());
        let fonts = Fonts::from_definitions(1., FontDefinitions::default());
        Self {
            render_pass: egui_rpass,
            tessellator,
            texture: fonts.texture(),
            fonts,
            paint_jobs: vec![],
        }
    }

    pub fn drawer<'p>(&'p mut self, clip_rectangle: Rect) -> UiDrawer<'p> {
        UiDrawer {
            render_pass: self,
            clipped_mesh: ClippedMesh(clip_rectangle, Mesh::default()),
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        window_size: (u32, u32),
        scale_factor: f32,
        clear_color: Option<wgpu::Color>,
    ) {
        let descriptor = ScreenDescriptor {
            physical_width: window_size.0,
            physical_height: window_size.1,
            scale_factor,
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("niobe-core::UiRenderPass"),
        });
        self.render_pass
            .update_texture(device, queue, &self.texture);
        self.render_pass.update_user_textures(device, queue);
        self.render_pass
            .update_buffers(device, queue, &self.paint_jobs, &descriptor);

        // Record all render passes.
        self.render_pass
            .execute(
                &mut encoder,
                view,
                &self.paint_jobs,
                &descriptor,
                clear_color,
            )
            .unwrap();
        self.paint_jobs.clear();
        queue.submit(iter::once(encoder.finish()));
    }
}

pub struct UiDrawer<'p> {
    render_pass: &'p mut UiRenderPass,
    clipped_mesh: ClippedMesh,
}

impl<'p> UiDrawer<'p> {
    pub fn circle(
        &mut self,
        center: Pos2,
        radius: f32,
        fill: Color32,
        stroke: Stroke,
    ) -> &mut Self {
        self.render_pass.tessellator.tessellate_shape(
            [1, 1],
            Shape::Circle {
                center,
                radius,
                fill,
                stroke,
            },
            &mut self.clipped_mesh.1,
        );
        self
    }

    pub fn line(&mut self, start: Pos2, end: Pos2, stroke: Stroke) -> &mut Self {
        self.render_pass.tessellator.tessellate_shape(
            [1, 1],
            Shape::LineSegment {
                points: [start, end],
                stroke,
            },
            &mut self.clipped_mesh.1,
        );
        self
    }

    pub fn path(
        &mut self,
        points: Vec<Pos2>,
        closed: bool,
        fill: Color32,
        stroke: Stroke,
    ) -> &mut Self {
        self.render_pass.tessellator.tessellate_shape(
            [1, 1],
            Shape::Path {
                points,
                closed,
                fill,
                stroke,
            },
            &mut self.clipped_mesh.1,
        );
        self
    }

    /// stroke is applied in the middle of possition
    /// meaning if we have a rect that is 100 by 100 at pos 100,100 and a stroke of 20
    /// border will start at 90,90, it will have a width of 20, then fill color will start at
    /// 110, 110
    pub fn rect(
        &mut self,
        rect: Rect,
        corner_radius: f32,
        fill: Color32,
        stroke: Stroke,
    ) -> &mut Self {
        self.render_pass.tessellator.tessellate_shape(
            [1, 1],
            Shape::Rect {
                rect,
                corner_radius,
                fill,
                stroke,
            },
            &mut self.clipped_mesh.1,
        );
        self
    }

    pub fn text_single_line(
        &mut self,
        style: TextStyle,
        text: impl Into<String>,
        pos: Pos2,
        color: Color32,
    ) -> &mut Self {
        let galley = self
            .render_pass
            .fonts
            .layout_single_line(style, text.into());
        self.render_pass.tessellator.tessellate_text(
            [
                self.render_pass.texture.width,
                self.render_pass.texture.height,
            ],
            pos,
            &*galley,
            color,
            false,
            &mut self.clipped_mesh.1,
        );
        self
    }
}
