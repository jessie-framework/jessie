use std::any::Any;

use miniquad::*;
pub struct MiniquadRenderer {
    renderer: Box<dyn RenderingBackend>,
}

impl MiniquadRenderer {
    pub fn new() -> Self {
        let renderer = window::new_rendering_backend();
        Self { renderer }
    }
}

struct ResizableBuffer<'a, T>
where
    T: Sized,
{
    buffer: BufferId,
    capacity: usize,
    data: Vec<T>,
    ctx: &'a mut Box<dyn RenderingBackend>,
    buffertype: BufferType,
}

impl<'a, T> ResizableBuffer<'a, T>
where
    T: Sized,
{
    fn new(
        capacity: usize,
        ctx: &'a mut Box<dyn RenderingBackend>,
        buffertype: BufferType,
    ) -> Self {
        Self {
            buffer: ctx.new_buffer(
                buffertype,
                BufferUsage::Stream,
                BufferSource::Empty {
                    size: capacity,
                    element_size: std::mem::size_of::<T>(),
                },
            ),
            capacity,
            data: Vec::new(),
            ctx,
            buffertype,
        }
    }

    fn push(&'a mut self, value: T) {
        if self.data.len() == self.capacity {
            self.ctx.delete_buffer(self.buffer);
            self.capacity *= 2;
            let new_buffer = self.ctx.new_buffer(
                self.buffertype,
                BufferUsage::Stream,
                BufferSource::Empty {
                    size: self.capacity,
                    element_size: std::mem::size_of::<T>(),
                },
            );
            self.buffer = new_buffer;
        }

        self.data.push(value);
    }

    fn upload(&'a mut self) {
        self.ctx
            .buffer_update(self.buffer, BufferSource::slice(self.data.as_slice()));
    }
}
