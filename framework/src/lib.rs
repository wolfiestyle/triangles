//TODO: replace GLenum with rust enums
//      use Gl with struct context
//      link referenced gl objects via rust lifetimes

pub mod types;
pub mod buffer;
pub mod shader;
pub mod texture;

#[cfg(feature = "glutin")]
pub mod ctx_glutin;

pub use crate::types::*;
pub use crate::buffer::*;
pub use crate::shader::*;
pub use crate::texture::*;

use std::ptr;
use std::ffi::CStr;
use gl::types::{GLenum, GLuint, GLsizei, GLchar, GLvoid};

#[allow(unused_variables)]
extern "system"
fn debug_callback(source: GLenum, ty: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, user_param: *mut GLvoid)
{
    let msg = unsafe{ CStr::from_ptr(message) };
    eprintln!("debug: {:?}", msg);
}

pub fn enable_debug_callback()
{
    unsafe
    {
        gl::DebugMessageCallback(Some(debug_callback), ptr::null());
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    }
}
