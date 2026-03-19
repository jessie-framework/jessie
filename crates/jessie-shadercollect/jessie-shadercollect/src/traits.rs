pub unsafe trait IntoAttrPtr {
    const ATTRPTR: crate::prelude::AttrPtrHandle;
}

pub unsafe trait IntoGlType {
    const GL_TYPE: u32;
}

pub unsafe trait IntoGlSize {
    const GL_SIZE: usize;
}

unsafe impl<const _N: usize> IntoGlType for [f32; _N] {
    const GL_TYPE: u32 = gl::FLOAT;
}
unsafe impl<const N: usize> IntoGlSize for [f32; N] {
    const GL_SIZE: usize = N;
}
