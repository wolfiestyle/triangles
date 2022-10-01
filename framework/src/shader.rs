use crate::types::UniformValue;
use gl::types::*;
use std::ffi::CString;
use std::ptr;

// shader/program validation
trait ShaderStatus {
    fn get_status(&self) -> bool;
    fn get_log(&self) -> Option<CString>;
}

fn validate_shader<T: ShaderStatus>(shader: T) -> Result<T, CString> {
    if shader.get_status() {
        shader.get_log().map(|log| eprintln!("-- {:?}", log));
        Ok(shader)
    } else {
        Err(shader.get_log().unwrap_or_else(|| CString::new("unknown error").unwrap()))
    }
}

fn get_shader_log(id: GLuint) -> Option<CString> {
    let mut log_len = 0;
    unsafe { gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut log_len) };
    if log_len > 0 {
        let mut log_buff = vec![0u8; log_len as usize];
        unsafe { gl::GetShaderInfoLog(id, log_len, ptr::null_mut(), log_buff.as_mut_ptr() as *mut _) };
        Some(CString::from_vec_with_nul(log_buff).unwrap())
    } else {
        None
    }
}

// shader object
#[derive(Debug)]
pub struct Shader(GLuint);

impl Shader {
    pub fn new(ty: GLenum, source: &[&str]) -> Result<Self, CString> {
        unsafe {
            let src: Vec<_> = source.iter().map(|s| s.as_ptr() as *const _).collect();
            let src_len: Vec<_> = source.iter().map(|s| s.len() as GLint).collect();
            let id = gl::CreateShader(ty);
            gl::ShaderSource(id, source.len() as GLsizei, src.as_ptr(), src_len.as_ptr());
            gl::CompileShader(id);

            validate_shader(Shader(id))
        }
    }
}

impl ShaderStatus for Shader {
    fn get_status(&self) -> bool {
        let mut status = 0;
        unsafe { gl::GetShaderiv(self.0, gl::COMPILE_STATUS, &mut status) };
        status != 0
    }

    fn get_log(&self) -> Option<CString> {
        get_shader_log(self.0)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) };
    }
}

// program object
#[derive(Debug)]
pub struct Program(GLuint);

impl Program {
    pub fn new(shaders: &[Shader]) -> Result<Program, CString> {
        unsafe {
            let id = gl::CreateProgram();

            for sh in shaders.iter() {
                gl::AttachShader(id, sh.0);
            }
            gl::LinkProgram(id);
            for sh in shaders.iter() {
                gl::DetachShader(id, sh.0);
            }

            validate_shader(Program(id))
        }
    }

    pub fn set_active(&self) {
        unsafe { gl::UseProgram(self.0) };
    }

    pub fn get_uniform(&self, name: &str) -> Option<Uniform> {
        let name_ = CString::new(name).unwrap();
        let id = unsafe { gl::GetUniformLocation(self.0, name_.as_ptr()) };
        if id < 0 {
            None
        } else {
            Some(Uniform { loc: id, prog: self })
        }
    }
}

impl ShaderStatus for Program {
    fn get_status(&self) -> bool {
        let mut status = 0;
        unsafe { gl::GetProgramiv(self.0, gl::LINK_STATUS, &mut status) };
        status != 0
    }

    fn get_log(&self) -> Option<CString> {
        get_shader_log(self.0)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.0) };
    }
}

// a program's uniform
#[derive(Debug)]
pub struct Uniform<'a> {
    loc: GLint,
    prog: &'a Program,
}

impl<'a> Uniform<'a> {
    pub fn set<T: UniformValue>(&self, value: T) {
        unsafe { value.write_uniform(self.prog.0, self.loc) };
    }
}
