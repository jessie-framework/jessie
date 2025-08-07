use crate::eventloop::renderer::Renderer;
use std::any::Any;

use miniquad::*;

#[repr(C)]
pub struct Quad {
    pos: (u32, u32),
    size: (u32, u32),
}
pub struct MiniquadRenderer {
    renderer: Box<dyn RenderingBackend>,
    quadbuf: ResizableBuffer<Quad>,
}

impl Renderer for MiniquadRenderer {
    fn init(&mut self) {}

    fn render(&mut self) {}
}

impl Default for MiniquadRenderer {
    fn default() -> Self {
        let mut renderer = window::new_rendering_backend();
        let quadbuf: ResizableBuffer<Quad> =
            ResizableBuffer::new(200, &mut renderer, BufferType::VertexBuffer);
        Self { renderer, quadbuf }
    }
}

impl<'a> EventHandler for MiniquadRenderer {
    fn draw(&mut self) {
        self.renderer.begin_default_pass(Default::default());
    }

    fn update(&mut self) {}
}

struct ResizableBuffer<T>
where
    T: Sized,
{
    buffer: BufferId,
    capacity: usize,
    data: Vec<T>,
    buffertype: BufferType,
}

impl<T> ResizableBuffer<T>
where
    T: Sized,
{
    fn new(capacity: usize, ctx: &mut Box<dyn RenderingBackend>, buffertype: BufferType) -> Self {
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
            buffertype,
        }
    }

    fn push(&mut self, value: T, ctx: &mut Box<dyn RenderingBackend>) {
        if self.data.len() == self.capacity {
            ctx.delete_buffer(self.buffer);
            self.capacity *= 2;
            let new_buffer = ctx.new_buffer(
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

    fn upload(&mut self, ctx: &mut Box<dyn RenderingBackend>) {
        ctx.buffer_update(self.buffer, BufferSource::slice(self.data.as_slice()));
    }
}
