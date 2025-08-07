#[cfg(feature = "miniquad")]
pub mod miniquad;

///A trait for renderers.
///The logic is simple : at the beginning of the event loop, the init function is called. After that, the render function is called every frame.
pub trait Renderer {
    fn run();
}

///Draw calls to the rendering backends.
pub enum DrawCall {
    DrawRect {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        color: (u8, u8, u8),
    },
}
