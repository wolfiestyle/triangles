use gl::types::*;

// mapping from rust type => opengl type enum
pub trait GlType
{
    fn get_gl_type() -> GLenum;
}

impl GlType for i8 { fn get_gl_type() -> GLenum { gl::BYTE } }
impl GlType for u8 { fn get_gl_type() -> GLenum { gl::UNSIGNED_BYTE } }
impl GlType for i16 { fn get_gl_type() -> GLenum { gl::SHORT } }
impl GlType for u16 { fn get_gl_type() -> GLenum { gl::UNSIGNED_SHORT } }
impl GlType for i32 { fn get_gl_type() -> GLenum { gl::INT } }
impl GlType for u32 { fn get_gl_type() -> GLenum { gl::UNSIGNED_INT } }
impl GlType for f32 { fn get_gl_type() -> GLenum { gl::FLOAT } }
impl GlType for f64 { fn get_gl_type() -> GLenum { gl::DOUBLE } }

impl<T> GlType for [T; 1] where T: GlType { fn get_gl_type() -> GLenum { T::get_gl_type() } }
impl<T> GlType for [T; 2] where T: GlType { fn get_gl_type() -> GLenum { T::get_gl_type() } }
impl<T> GlType for [T; 3] where T: GlType { fn get_gl_type() -> GLenum { T::get_gl_type() } }
impl<T> GlType for [T; 4] where T: GlType { fn get_gl_type() -> GLenum { T::get_gl_type() } }

impl<'a, T> GlType for &'a T where T: GlType { fn get_gl_type() -> GLenum { T::get_gl_type() } }

// mapping from rust type => glUniformX
pub trait UniformValue
{
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint);
}

impl UniformValue for i32
{
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint)
    {
        gl::ProgramUniform1i(prog, loc, self);
    }
}

impl UniformValue for u32
{
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint)
    {
        gl::ProgramUniform1ui(prog, loc, self);
    }
}
