#[derive(Debug)]
pub struct ProgramHandle
where
    Self: 'static,
{
    pub vs: &'static VertexShaderHandle,
    pub fs: &'static FragmentShaderHandle,
    pub attr_ptr: AttrPtrHandle,
}

#[derive(Debug)]
pub struct VertexShaderHandle(pub &'static std::ffi::CStr)
where
    Self: 'static;

#[derive(Debug)]
pub struct FragmentShaderHandle(pub &'static std::ffi::CStr)
where
    Self: 'static;

#[derive(Debug)]
pub struct AttrPtrHandle
where
    Self: 'static,
{
    pub stride: i32,
    pub attr_count: usize,
    pub locs: &'static [LocationHandle],
}

#[derive(Debug)]
pub struct LocationHandle
where
    Self: 'static,
{
    pub name: &'static std::ffi::CStr,
    pub offset: i32,
    pub gl_type: u32,
    pub size: usize,
}
