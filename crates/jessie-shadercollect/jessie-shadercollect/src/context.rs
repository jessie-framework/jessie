use std::ptr::null;

use crate::handle::FragmentShaderHandle;
use crate::handle::VertexShaderHandle;
use crate::section;
pub struct Context {
    vertex_shaders: Vec<u32>,
    fragment_shaders: Vec<u32>,
    programs: Vec<u32>,
}

impl Context {
    /// Creates a new context.
    /// # Safety
    /// This function deals with raw pointers and OpenGL calls. It WILL break if any program or shader was not declared according to convention.
    pub unsafe fn generate() -> Self {
        let fragment_handles = unsafe {
            make_slice(
                &section::FRAGMENT_SHADER_START,
                &section::FRAGMENT_SHADER_STOP,
            )
        };
        let vertex_handles =
            unsafe { make_slice(&section::VERTEX_SHADER_START, &section::VERTEX_SHADER_STOP) };
        let program_handles =
            unsafe { make_slice(&section::PROGRAM_START, &section::PROGRAM_STOP) };
        let mut fragment_shaders = Vec::with_capacity(fragment_handles.len());
        let mut vertex_shaders = Vec::with_capacity(vertex_handles.len());
        let mut programs = Vec::with_capacity(program_handles.len());
        for handle in fragment_handles {
            unsafe {
                let shader = gl::CreateShader(gl::FRAGMENT_SHADER);
                gl::ShaderSource(shader, 1, &raw const handle.0 as *const _, null());
                gl::CompileShader(shader);
                fragment_shaders.push(shader);
            }
        }
        for handle in vertex_handles {
            unsafe {
                let shader = gl::CreateShader(gl::VERTEX_SHADER);
                gl::ShaderSource(shader, 1, &raw const handle.0 as *const _, null());
                gl::CompileShader(shader);
                vertex_shaders.push(shader);
            }
        }

        for handle in program_handles {
            unsafe {
                let program = gl::CreateProgram();
                let fragment_offset = {
                    (handle.fs as *const FragmentShaderHandle)
                        .offset_from(&raw const section::FRAGMENT_SHADER_START)
                        as usize
                };
                let vertex_offset = {
                    (handle.vs as *const VertexShaderHandle)
                        .offset_from(&raw const section::VERTEX_SHADER_START)
                        as usize
                };
                gl::AttachShader(program, fragment_shaders[fragment_offset]);
                gl::AttachShader(program, vertex_shaders[vertex_offset]);
                for (i, h) in handle.attr_ptr.locs.iter().enumerate() {
                    gl::BindAttribLocation(program, i as u32, &raw const h.name as *const _);
                }
                gl::LinkProgram(program);
                programs.push(program);
            }
        }
        Self {
            vertex_shaders,
            fragment_shaders,
            programs,
        }
    }
}

unsafe fn make_slice<T>(start: &'static T, stop: &'static T) -> &'static [T] {
    let begin_ptr: *const T = start as *const _;
    let stop_ptr: *const T = stop as *const _;
    unsafe {
        let len = stop_ptr.offset_from(begin_ptr) as usize;
        core::slice::from_raw_parts(begin_ptr, len)
    }
}
