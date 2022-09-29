use gl::types::*;

macro_rules! impl_gltype {
    ($rust:ty, $gl:expr) => {
        impl GlType for $rust {
            fn get_gl_type() -> GLenum {
                $gl
            }
        }
    };
}

// mapping from rust type => opengl type enum
pub trait GlType {
    fn get_gl_type() -> GLenum;
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
}

impl<'a, T> GlType for &'a T
where
    T: GlType,
{
    fn get_gl_type() -> GLenum {
        T::get_gl_type()
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

// returns the number of components of a pixel format
pub fn pixel_format_components(format: GLenum) -> usize {
    match format {
        gl::STENCIL_INDEX
        | gl::DEPTH_COMPONENT
        | gl::RED
        | gl::GREEN
        | gl::BLUE
        | gl::RED_INTEGER
        | gl::GREEN_INTEGER
        | gl::BLUE_INTEGER => 1,
        gl::DEPTH_STENCIL | gl::RG | gl::RG_INTEGER => 2,
        gl::RGB | gl::BGR | gl::RGB_INTEGER | gl::BGR_INTEGER => 3,
        gl::RGBA | gl::BGRA | gl::RGBA_INTEGER | gl::BGRA_INTEGER => 4,
        _ => panic!("invalid pixel format"),
    }
}
