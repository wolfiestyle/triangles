//TODO: replace GLenum with rust enums
//      use Gl with struct context
//      link referenced gl objects via rust lifetimes
pub extern crate custom_gl as gl;

#[cfg(feature = "glutin")]
pub extern crate glutin;
#[cfg(feature = "image")]
pub extern crate image;

pub mod types;
pub mod buffer;
pub mod shader;
pub mod texture;

#[cfg(feature = "glutin")]
pub mod ctx_glutin;

pub use types::*;
pub use buffer::*;
pub use shader::*;
pub use texture::*;

use gl::types::*;
use std::ptr;
use std::ffi::CStr;

#[allow(unused_variables)]
extern "system"
fn debug_callback(source: GLenum, ty: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, user_param: *mut GLvoid)
{
    let msg = unsafe{ CStr::from_ptr(message) };
    println!("debug: {:?}", msg);
}

pub fn enable_debug_callback()
{
    unsafe
    {
        gl::DebugMessageCallback(debug_callback, ptr::null());
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    }
}
