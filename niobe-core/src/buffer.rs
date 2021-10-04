use bytemuck::Pod;
use num_traits::{CheckedAdd, One, Zero};
use std::collections::Bound;
use std::mem;
use std::ops::{Index, Range, RangeBounds};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferAddress, BufferUsages, Device, Queue};

#[derive(Debug)]
pub struct Buffer<T: Copy + Pod> {
    pub buf: wgpu::Buffer,
    len: u32,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: Copy + Pod> Buffer<T> {
    pub fn new(device: &Device, usage: BufferUsages, data: &[T]) -> Self {
        let contents = bytemuck::cast_slice(data);
        debug_assert!(data.len() <= u32::MAX as usize);

        Self {
            buf: device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents,
                usage,
            }),
            len: data.len() as u32,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn slice(&self, range: impl RangeBounds<u32>) -> BufferSlice<'_, T> {
        let start = match range.start_bound() {
            Bound::Included(b) => *b,
            Bound::Excluded(_) => unreachable!("range start bound cannot be excluded"),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(b) => *b + 1,
            Bound::Excluded(b) => *b,
            Bound::Unbounded => self.len,
        };
        BufferSlice {
            buf: &self.buf,
            range: start..end,
            phantom_data: Default::default(),
        }
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn write(&self, queue: &Queue, offset: usize, data: &[T]) {
        queue.write_buffer(
            &self.buf,
            (offset * mem::size_of::<T>()) as BufferAddress,
            bytemuck::cast_slice(data),
        )
    }

    pub fn write_sliced(&self, queue: &Queue, range: impl RangeBounds<usize>, data: &[T]) {
        let range = range_from_range_bounds(range, data.len());
        queue.write_buffer(
            &self.buf,
            (range.start * mem::size_of::<T>()) as BufferAddress,
            bytemuck::cast_slice(&data[range]),
        )
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

#[derive(Clone)]
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

    pub fn len(&self) -> u32 {
        self.range.end - self.range.start
    }
}

fn range_from_range_bounds<T: One + CheckedAdd + Zero + Copy>(
    bounds: impl RangeBounds<T>,
    max_bound: T,
) -> Range<T> {
    let start = match bounds.start_bound() {
        Bound::Included(b) => *b,
        Bound::Excluded(_) => unreachable!("range start bound cannot be excluded"),
        Bound::Unbounded => T::zero(),
    };
    let end = match bounds.end_bound() {
        Bound::Included(b) => b.checked_add(&T::one()).unwrap(),
        Bound::Excluded(b) => *b,
        Bound::Unbounded => max_bound,
    };
    start..end
}
