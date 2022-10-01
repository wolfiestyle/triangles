use gl::types::*;

macro_rules! impl_gltype {
    ($rust:ty, $gl:expr) => {
        impl GlType for $rust {
            fn get_gl_type() -> GLenum {
                $gl
            }
            fn num_components() -> usize {
                1
            }
        }
    };
}

// mapping from rust type => opengl type enum
pub trait GlType {
    fn get_gl_type() -> GLenum;
    fn num_components() -> usize;
}

impl_gltype!(i8, gl::BYTE);
impl_gltype!(u8, gl::UNSIGNED_BYTE);
impl_gltype!(i16, gl::SHORT);
impl_gltype!(u16, gl::UNSIGNED_SHORT);
impl_gltype!(i32, gl::INT);
impl_gltype!(u32, gl::UNSIGNED_INT);
impl_gltype!(f32, gl::FLOAT);
impl_gltype!(f64, gl::DOUBLE);

impl<T, const N: usize> GlType for [T; N]
where
    T: GlType,
{
    fn get_gl_type() -> GLenum {
        T::get_gl_type()
    }

    fn num_components() -> usize {
        N
    }
}

impl<'a, T> GlType for &'a T
where
    T: GlType,
{
    fn get_gl_type() -> GLenum {
        T::get_gl_type()
    }

    fn num_components() -> usize {
        T::num_components()
    }
}

// mapping from rust type => glUniformX
pub trait UniformValue {
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint);
}

impl UniformValue for i32 {
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint) {
        gl::ProgramUniform1i(prog, loc, self);
    }
}

impl UniformValue for u32 {
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint) {
        gl::ProgramUniform1ui(prog, loc, self);
    }
}
