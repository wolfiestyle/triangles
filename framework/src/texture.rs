use crate::{
    pixel_format_components,
    types::{GlType, UniformValue},
};
use gl::types::*;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

#[cfg(feature = "image")]
use image::{imageops::FilterType, GenericImageView};

#[derive(Debug)]
pub struct Texture2d {
    id: GLuint,
    width: u32,
    height: u32,
}

impl Texture2d {
    pub fn new(width: u32, height: u32, int_format: GLenum) -> Self {
        let mut id = 0;
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);
            gl::TextureStorage2D(id, 1, int_format, width as GLsizei, height as GLsizei);
        }
        Texture2d { id, width, height }
    }

    #[cfg(feature = "image")]
    pub fn from_image(image: image::DynamicImage, int_format: GLenum) -> Self {
        let (width, height) = image.dimensions();
        let tex = Texture2d::new(width, height, int_format);
        tex.load_data(0, 0, width, height, gl::RGBA, image.into_rgba8().as_raw());
        tex
    }

    #[cfg(feature = "image")]
    pub fn load_image(&self, image: image::DynamicImage) {
        let dims = image.dimensions();
        let resized = if dims != (self.width, self.height) {
            image.resize_exact(self.width, self.height, FilterType::Lanczos3)
        } else {
            image
        };
        self.load_data(0, 0, self.width, self.height, gl::RGBA, resized.into_rgba8().as_raw());
    }

    pub fn load_data<T: GlType>(&self, x: i32, y: i32, width: u32, height: u32, pix_format: GLenum, data: &[T]) {
        assert_eq!(
            // here T is a single color component (like u8)
            width as usize * height as usize * pixel_format_components(pix_format) * mem::size_of::<T>(),
            mem::size_of_val(data)
        );
        unsafe {
            gl::TextureSubImage2D(
                self.id,
                0,
                x,
                y,
                width as GLsizei,
                height as GLsizei,
                pix_format,
                T::get_gl_type(),
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn read_data<T: GlType>(&self, format: GLenum) -> Vec<T>
    where
        T: Default + Clone,
    {
        let fmt_components = pixel_format_components(format);
        assert_eq!(fmt_components % T::num_components(), 0);
        let elem_per_pix = fmt_components / T::num_components();
        let n_elems = self.width as usize * self.height as usize * elem_per_pix;
        let size = n_elems * mem::size_of::<T>();
        let mut buf = vec![T::default(); n_elems];
        unsafe {
            gl::GetTextureImage(self.id, 0, format, T::get_gl_type(), size as GLsizei, buf.as_mut_ptr() as *mut _);
        }
        buf
    }

    pub fn set_filter(&self, min: GLenum, mag: GLenum) {
        unsafe {
            gl::TextureParameteri(self.id, gl::TEXTURE_MIN_FILTER, min as GLint);
            gl::TextureParameteri(self.id, gl::TEXTURE_MAG_FILTER, mag as GLint);
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn bind_to(&self, tex_unit: GLuint) {
        unsafe { gl::BindTextureUnit(tex_unit, self.id) };
    }

    pub fn bind_to_image(&self, img_unit: GLuint, level: GLint, format: GLenum, access: GLenum) {
        unsafe { gl::BindImageTexture(img_unit, self.id, level, gl::FALSE, 0, access, format) };
    }

    pub fn into_bindless(self) -> Bindless<Self> {
        unsafe {
            let handle = gl::GetTextureHandleARB(self.id);
            gl::MakeTextureHandleResidentARB(handle);
            Bindless { handle, obj: self }
        }
    }

    pub fn as_image(&self, level: GLint, format: GLenum, access: GLenum) -> Image<Self> {
        unsafe {
            let handle = gl::GetImageHandleARB(self.id, level, gl::FALSE, 0, format);
            gl::MakeImageHandleResidentARB(handle, access);
            Image(handle, PhantomData)
        }
    }

    pub fn into_framebuffer(self) -> Result<TexFramebuffer, String> {
        let fbo = Framebuffer::new();
        fbo.attach_texture(gl::COLOR_ATTACHMENT0, &self);
        fbo.bind_locations(&[gl::COLOR_ATTACHMENT0]);
        fbo.validate().and(Ok(TexFramebuffer { fbo, tex: self }))
    }
}

impl Drop for Texture2d {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) };
    }
}

// bindless image handle
#[derive(Debug)]
pub struct Image<'a, T: 'a>(u64, PhantomData<&'a T>);

impl<'a, T> UniformValue for &'a Image<'a, T> {
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint) {
        gl::ProgramUniformHandleui64ARB(prog, loc, self.0);
    }
}

impl<'a, T> Drop for Image<'a, T> {
    fn drop(&mut self) {
        unsafe { gl::MakeImageHandleNonResidentARB(self.0) };
    }
}

// bindless texture
#[derive(Debug)]
pub struct Bindless<T> {
    handle: u64,
    obj: T,
}

impl Bindless<Texture2d> {
    pub fn into_non_resident(self) -> Texture2d {
        unsafe { gl::MakeTextureHandleNonResidentARB(self.handle) };
        self.obj
    }
}

impl<'a, T> UniformValue for &'a Bindless<T> {
    unsafe fn write_uniform(self, prog: GLuint, loc: GLint) {
        gl::ProgramUniformHandleui64ARB(prog, loc, self.handle);
    }
}

impl<T> Deref for Bindless<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

// framebuffer object
//FIXME: this probably should use a builder object
#[derive(Debug)]
pub struct Framebuffer(GLuint);

impl Framebuffer {
    pub fn new() -> Self {
        unsafe {
            let mut id = 0;
            gl::CreateFramebuffers(1, &mut id);
            Framebuffer(id)
        }
    }

    pub fn attach_texture(&self, attachment: GLenum, texture: &Texture2d) {
        unsafe { gl::NamedFramebufferTexture(self.0, attachment, texture.id, 0) };
    }

    pub fn bind_locations(&self, locations: &[GLenum]) {
        unsafe { gl::NamedFramebufferDrawBuffers(self.0, locations.len() as GLsizei, locations.as_ptr()) };
    }

    pub fn validate(&self) -> Result<(), String> {
        let status = unsafe { gl::CheckNamedFramebufferStatus(self.0, gl::FRAMEBUFFER) };
        match status {
            gl::FRAMEBUFFER_COMPLETE => Ok(()),
            code => Err(format!("invalid Framebuffer: {:x}", code)),
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.0) };
    }

    pub fn unbind() {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.0) };
    }
}

// framebuffer bound to a single texture
#[derive(Debug)]
pub struct TexFramebuffer {
    fbo: Framebuffer,
    tex: Texture2d,
}

impl TexFramebuffer {
    pub fn get_tex(&self) -> &Texture2d {
        &self.tex
    }
}

impl Deref for TexFramebuffer {
    type Target = Framebuffer;

    fn deref(&self) -> &Self::Target {
        &self.fbo
    }
}
