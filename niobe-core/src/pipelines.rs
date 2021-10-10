use std::ops::{Deref, DerefMut};
use wgpu::{CommandEncoder, Queue, RenderPass, RenderPipeline};

pub mod line;
pub mod mesh;
pub mod ui;

pub struct Drawer<'a> {
    pub pipeline: &'a RenderPipeline,
    pub pass: &'a mut RenderPass<'a>,
}
