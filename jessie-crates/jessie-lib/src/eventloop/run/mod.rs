use crate::appinfo::AppInfo;
use crate::eventloop::renderer::Renderer;
use crate::layoutinfo::LayoutInfo;
use crate::layoutprovider::LayoutProvider;

use super::renderer::miniquad::MiniquadRenderer;
pub fn run(app: AppInfo, mainexpectation: impl LayoutProvider) {
    #[cfg(feature = "miniquad")]
    let mut renderer = MiniquadRenderer::default();

    renderer.init();
    loop {
        renderer.render();
    }
}
