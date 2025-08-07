#[cfg(feature = "miniquad")]
pub mod miniquad;

///A trait for renderers.
///The logic is simple : at the beginning of the event loop, the init function is called. After that, the render function is called every frame.
pub trait Renderer {
    fn run();
}
