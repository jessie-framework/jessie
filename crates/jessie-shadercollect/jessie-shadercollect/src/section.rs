use crate::handle;
unsafe extern "Rust" {
    #[linkeasy::start("sc_programs")]
    pub(crate) static PROGRAM_START: handle::ProgramHandle;
    #[linkeasy::stop("sc_programs")]
    pub(crate) static PROGRAM_STOP: handle::ProgramHandle;
    #[linkeasy::start("sc_vertex_shaders")]
    pub(crate) static VERTEX_SHADER_START: handle::VertexShaderHandle;
    #[linkeasy::stop("sc_vertex_shaders")]
    pub(crate) static VERTEX_SHADER_STOP: handle::VertexShaderHandle;
    #[linkeasy::start("sc_fragment_shaders")]
    pub(crate) static FRAGMENT_SHADER_START: handle::FragmentShaderHandle;
    #[linkeasy::stop("sc_fragment_shaders")]
    pub(crate) static FRAGMENT_SHADER_STOP: handle::FragmentShaderHandle;
}
