#[macro_use]
extern crate bytemuck;
#[macro_use]
extern crate derive_more;

use crate::buffer::Buffer;
use bytemuck::Pod;
use nalgebra_glm::Vec2;

pub mod buffer;
pub mod pipelines;

#[rustfmt::skip]
pub mod colors;

pub type Point2d = Vec2;

pub struct Mesh2d<IF: IndexFormat> {
    pub vertices: Buffer<Point2d>,
    pub indices: Buffer<IF>,
}

pub trait IndexFormat: Pod {
    const FORMAT: wgpu::IndexFormat;
}
impl IndexFormat for u16 {
    const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
}
impl IndexFormat for u32 {
    const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
}

pub trait Point2dExt {
    fn all(value: f32) -> Point2d;
}

impl Point2dExt for Point2d {
    fn all(value: f32) -> Point2d {
        Point2d::new(value, value)
    }
}
