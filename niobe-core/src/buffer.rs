use bytemuck::Pod;
use std::mem;
use std::ops::Range;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, BufferUsages, Device};

pub struct Buffer<T: Copy + Pod> {
    pub(super) buf: wgpu::Buffer,
    len: usize,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: Copy + Pod> Buffer<T> {
    pub fn new(device: &Device, usage: BufferUsages, data: &[T]) -> Self {
        let contents = bytemuck::cast_slice(data);

        Self {
            buf: device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents,
                usage,
            }),
            len: data.len(),
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn slice(&self, range: Range<u32>) -> BufferSlice<'_, T> {
        BufferSlice {
            buf: &self.buf,
            range,
            phantom_data: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

//pub struct DynamicBuffer<T: Copy + Pod>(Buffer<T>);
//
//impl<T: Copy + Pod> DynamicBuffer<T> {
//    pub fn new(device: &wgpu::Device, len: usize, usage: wgpu::BufferUsage) -> Self {
//        let buffer = Buffer {
//            buf: device.create_buffer(&wgpu::BufferDescriptor {
//                label: None,
//                mapped_at_creation: false,
//                size: len as u64 * std::mem::size_of::<T>() as u64,
//                usage: usage | wgpu::BufferUsage::COPY_DST,
//            }),
//            len,
//            phantom_data: std::marker::PhantomData,
//        };
//        Self(buffer)
//    }
//
//    pub fn update(&self, queue: &wgpu::Queue, vals: &[T], offset: usize) {
//        if !vals.is_empty() {
//            queue.write_buffer(
//                &self.buf,
//                offset as u64 * std::mem::size_of::<T>() as u64,
//                bytemuck::cast_slice(vals),
//            )
//        }
//    }
//}
//
//impl<T: Copy + Pod> std::ops::Deref for DynamicBuffer<T> {
//    type Target = Buffer<T>;
//
//    fn deref(&self) -> &Self::Target { &self.0 }
//}

pub struct BufferSlice<'a, T: Copy + Pod> {
    buf: &'a wgpu::Buffer,
    range: Range<u32>,
    phantom_data: std::marker::PhantomData<T>,
}

impl<'a, T: Copy + Pod> BufferSlice<'a, T> {
    pub fn raw_addres_range(&self) -> Range<BufferAddress> {
        let size = mem::size_of::<T>() as BufferAddress;
        self.range.start as BufferAddress * size..self.range.end as BufferAddress * size
    }

    pub fn range(&self) -> Range<u32> {
        self.range.clone()
    }

    pub fn buf(&self) -> &'a wgpu::Buffer {
        self.buf
    }

    pub fn to_raw_slice(&self) -> wgpu::BufferSlice<'a> {
        self.buf.slice(self.raw_addres_range())
    }
}
