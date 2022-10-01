//TODO: replace GLenum with rust enums
//      use Gl with struct context
//      link referenced gl objects via rust lifetimes

pub mod buffer;
pub mod shader;
pub mod texture;
pub mod types;

#[cfg(feature = "glutin")]
pub mod ctx_glutin;

pub use crate::buffer::*;
pub use crate::shader::*;
pub use crate::texture::*;
pub use crate::types::*;

use gl::types::{GLchar, GLenum, GLsizei, GLuint, GLvoid};
use std::ffi::CStr;
use std::ptr;

#[allow(unused_variables)]
extern "system" fn debug_callback(
    source: GLenum, ty: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, user_param: *mut GLvoid,
) {
    let msg = unsafe { CStr::from_ptr(message) };
    eprintln!("debug: {:?}", msg);
}

pub fn enable_debug_callback() {
    unsafe {
        gl::DebugMessageCallback(Some(debug_callback), ptr::null());
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    }
}

pub fn get_error_str() -> Option<&'static str> {
    let err = unsafe { gl::GetError() };
    Some(match err {
        gl::NO_ERROR => return None,
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        _ => "(unknown)",
    })
}
