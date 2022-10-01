use crate::types::GlType;
use gl::types::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::rc::Rc;

// vertex array object
pub struct VertexArray {
    id: GLuint,
    refs: HashMap<u32, Rc<dyn BufferAny>>, // we need this to own the buffers
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe { gl::CreateVertexArrays(1, &mut id) };
        VertexArray { id, refs: HashMap::new() }
    }

    pub fn set_attribute<T: GlType + 'static>(&mut self, attr_id: u32, vbo: Rc<Buffer<T>>, elem_count: u32, offset: usize, stride: usize) {
        let size = mem::size_of::<T>();
        let ty = T::get_gl_type();
        unsafe {
            gl::EnableVertexArrayAttrib(self.id, attr_id);
            gl::VertexArrayVertexBuffer(self.id, attr_id, vbo.id, 0, (stride * size) as GLsizei);
            gl::VertexArrayAttribFormat(self.id, attr_id, elem_count as GLint, ty, gl::FALSE, (offset * size) as GLuint);
            gl::VertexArrayAttribBinding(self.id, attr_id, attr_id);
        }
        self.refs.insert(attr_id, vbo);
    }

    pub fn draw(&self, mode: GLenum, first: u32, count: u32) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawArrays(mode, first as GLint, count as GLsizei);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id) };
    }
}

pub trait BufferAny {
    fn empty() -> Self
    where
        Self: Sized;

    fn alloc_bytes(&mut self, size: usize, usage: GLenum);

    fn byte_size(&self) -> usize;
}

// vertex buffer object
#[derive(Debug)]
pub struct Buffer<T> {
    id: GLuint,
    size: usize,
    usage: GLenum,
    _t: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(usage: GLenum, data: &[T]) -> Self {
        let mut buf = Buffer::empty();
        buf.load(usage, data);
        buf
    }

    pub fn load(&mut self, usage: GLenum, data: &[T]) {
        self.size = mem::size_of_val(data);
        self.usage = usage;
        unsafe { gl::NamedBufferData(self.id, self.size as GLsizeiptr, data.as_ptr() as *const _, usage) };
    }

    pub fn alloc(&mut self, n_elem: usize, usage: GLenum) {
        self.alloc_bytes(n_elem * mem::size_of::<T>(), usage)
    }

    pub fn read(&self, offset: usize, len: usize) -> Vec<T>
    where
        T: Default + Clone,
    {
        let buf = vec![T::default(); len];
        let off_bytes = offset * mem::size_of::<T>();
        let size = len * mem::size_of::<T>();
        unsafe { gl::GetNamedBufferSubData(self.id, off_bytes as GLintptr, size as GLsizeiptr, buf.as_ptr() as *mut _) };
        buf
    }

    pub fn write(&self, offset: usize, data: &[T]) {
        let byte_off = offset * mem::size_of::<T>();
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                byte_off as GLintptr,
                mem::size_of_val(data) as GLsizeiptr,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn get(&self, idx: usize) -> T
    where
        T: Default,
    {
        let size = mem::size_of::<T>();
        let idx_bytes = idx * size;
        let mut val = T::default();
        unsafe { gl::GetNamedBufferSubData(self.id, idx_bytes as GLintptr, size as GLsizeiptr, mem::transmute(&mut val)) };
        val
    }

    pub fn set(&self, idx: usize, val: T) {
        let size = mem::size_of::<T>();
        let idx_bytes = idx * size;
        unsafe { gl::NamedBufferSubData(self.id, idx_bytes as GLintptr, size as GLsizeiptr, mem::transmute(&val)) };
    }

    pub fn copy_into(&self, dest: &Buffer<T>, src_idx: usize, dst_idx: usize, len: usize) {
        let src_offset = src_idx * mem::size_of::<T>();
        let dst_offset = dst_idx * mem::size_of::<T>();
        let byte_len = len * mem::size_of::<T>();
        unsafe {
            gl::CopyNamedBufferSubData(
                self.id,
                dest.id,
                src_offset as GLsizeiptr,
                dst_offset as GLsizeiptr,
                byte_len as GLsizeiptr,
            )
        };
    }

    pub fn len(&self) -> usize {
        self.size / mem::size_of::<T>()
    }
}

impl<T> BufferAny for Buffer<T> {
    fn empty() -> Self {
        unsafe {
            let mut id = 0;
            gl::CreateBuffers(1, &mut id);
            Buffer {
                id,
                size: 0,
                usage: 0,
                _t: PhantomData,
            }
        }
    }

    fn alloc_bytes(&mut self, size: usize, usage: GLenum) {
        self.size = size;
        self.usage = usage;
        unsafe { gl::NamedBufferData(self.id, size as GLsizeiptr, ptr::null(), usage) };
    }

    fn byte_size(&self) -> usize {
        self.size
    }
}

impl<T> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        let mut dest = Buffer::empty();
        dest.alloc_bytes(self.size, self.usage);
        unsafe { gl::CopyNamedBufferSubData(self.id, dest.id, 0, 0, self.size as GLsizeiptr) };
        dest
    }

    fn clone_from(&mut self, source: &Self) {
        if self.size != source.size || self.usage != source.usage {
            self.alloc_bytes(source.size, source.usage);
        }
        unsafe { gl::CopyNamedBufferSubData(source.id, self.id, 0, 0, source.size as GLsizeiptr) };
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) };
    }
}
