use std::rc::Rc;

use GL::HasContext;

use crate::gcx::GL;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BufferType {
    ArrayBuffer = GL::ARRAY_BUFFER,
    ElementArrayBuffer = GL::ELEMENT_ARRAY_BUFFER,
    UniformBuffer = GL::UNIFORM_BUFFER,
    ShaderStorage = GL::SHADER_STORAGE_BUFFER,
}

#[derive(Debug)]
pub struct BufferInner {
    pub(super) gl: Rc<glow::Context>,
    pub(super) buffer: GL::Buffer,
    pub(super) ty: BufferType,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}

impl Buffer {
    pub fn update<T: bytemuck::NoUninit>(&mut self, offset: i32, data: &[T]) {
        let gl = &self.inner.gl;
        let ty = self.inner.ty as u32;
        unsafe {
            gl.bind_buffer(ty, Some(self.inner.buffer));
            gl.buffer_sub_data_u8_slice(ty, offset, bytemuck::cast_slice(data));
            gl.bind_buffer(ty, None);
        }
    }

    pub fn read(&mut self, offset: i32, length: i32, read: impl FnOnce(MapRead)) {
        let gl = &self.inner.gl;
        let ty = self.inner.ty as u32;
        unsafe {
            gl.bind_buffer(ty, Some(self.inner.buffer));
            let ptr = gl.map_buffer_range(ty, offset, length, GL::MAP_READ_BIT);
            if !ptr.is_null() {
                read(MapRead {
                    ptr,
                    size: length as usize,
                })
            }
            gl.bind_buffer(ty, None);
        }
    }

    pub fn write(&mut self, offset: i32, length: i32, write: impl FnOnce(MapWrite)) {
        let gl = &self.inner.gl;
        unsafe {
            gl.bind_buffer(self.inner.ty as u32, Some(self.inner.buffer));
            let ptr = gl.map_buffer_range(self.inner.ty as u32, offset, length, GL::MAP_WRITE_BIT);
            if !ptr.is_null() {
                write(MapWrite {
                    ptr,
                    size: length as usize,
                })
            }
            gl.bind_buffer(self.inner.ty as u32, None);
        }
    }

    pub(super) fn bind(&self) {
        unsafe {
            self.inner
                .gl
                .bind_buffer(self.inner.ty as u32, Some(self.inner.buffer));
        }
    }

    pub fn ty(&self) -> BufferType {
        self.inner.ty
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub(super) inner: Rc<BufferInner>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct BufferUsage: u32 {
        const DRAW_STREAM = GL::STREAM_DRAW;
        const DRAW_STATIC = GL::STATIC_DRAW;
        const DRAW_DYNAMIC = GL::DYNAMIC_DRAW;
    }

}

pub struct MapRead {
    ptr: *mut u8,
    size: usize,
}

impl MapRead {
    pub fn read<T: bytemuck::AnyBitPattern>(&self) -> &[T] {
        let slice = unsafe { core::slice::from_raw_parts(self.ptr, self.size) };
        bytemuck::cast_slice(slice)
    }
}

pub struct MapWrite {
    ptr: *mut u8,
    size: usize,
}

impl MapWrite {
    pub fn write<T: bytemuck::AnyBitPattern + bytemuck::NoUninit>(&mut self) -> &mut [T] {
        let slice = unsafe { core::slice::from_raw_parts_mut(self.ptr, self.size) };
        bytemuck::cast_slice_mut(slice)
    }
}
