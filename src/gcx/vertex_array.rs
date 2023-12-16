use GL::HasContext;

use super::{
    buffer::{Buffer, BufferType},
    texture::Texture,
    GCX, GL,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct VertexArray {
    pub(super) gl: Rc<glow::Context>,
    pub(super) vao: GL::VertexArray,

    // pub textures: Vec<Texture>,
    pub array_buffer: Buffer,
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DataType {
    F32 = GL::FLOAT,
}

pub struct Field {
    pub name: &'static str,
    pub ty: core::any::TypeId,
    pub size: i32,
    pub gl_size: i32,
    pub base: DataType,
}

impl Field {
    pub fn new<T: 'static + GLType>(name: &'static str) -> Self {
        Self {
            name,
            ty: core::any::TypeId::of::<T>(),
            size: core::mem::size_of::<T>() as i32,
            gl_size: T::size(),
            base: T::base(),
        }
    }
}

impl Fields for () {
    fn fields() -> Vec<Field> {
        Vec::with_capacity(0)
    }
}

impl<const SIZE: usize> Fields for [f32; SIZE] {
    fn fields() -> Vec<Field> {
        vec![Field::new::<[f32; SIZE]>("position")]
    }
}

pub trait Fields {
    fn fields() -> Vec<Field>;
}

pub trait GLType {
    fn base() -> DataType;
    fn size() -> i32;
}

impl GLType for f32 {
    fn base() -> DataType {
        DataType::F32
    }

    fn size() -> i32 {
        1
    }
}

impl<const SIZE: usize> GLType for [f32; SIZE] {
    fn base() -> DataType {
        DataType::F32
    }

    fn size() -> i32 {
        SIZE as i32
    }
}

pub struct AttribPointer {
    pub ty: DataType,
    pub size: i32,
    pub normalized: bool,
    pub stride: i32,
    pub offset: i32,
}

impl AttribPointer {
    pub fn new(ty: DataType, size: i32, stride: i32, normalized: bool, offset: i32) -> Self {
        Self {
            ty,
            size,
            normalized,
            stride,
            offset,
        }
    }

    pub fn stride(&self) -> i32 {
        self.stride
    }
}

pub struct VertexArrayBuilder<T: Fields> {
    pub(super) array_buffer: Buffer,

    pub(super) attribs: Vec<AttribPointer>,
    pub(super) _marker: core::marker::PhantomData<T>,
}

impl<T: Fields> VertexArrayBuilder<T> {
    pub fn add_buffer(mut self, buffer: Buffer) -> Self {
        match buffer.ty() {
            BufferType::ArrayBuffer => {
                self.array_buffer = buffer;
            }
            BufferType::ElementArrayBuffer => todo!(),
            BufferType::UniformBuffer => todo!(),
            BufferType::ShaderStorage => todo!(),
        }
        self
    }

    pub fn add_attrib(mut self, attrib: AttribPointer) -> Self {
        self.attribs.push(attrib);
        self
    }

    pub fn build(mut self, gcx: &GCX) -> VertexArray {
        unsafe {
            let array_buffer = self.array_buffer;

            let gl = &gcx.gl;
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            array_buffer.bind();

            if self.attribs.is_empty() {
                let mut stride = 0;
                for field in T::fields() {
                    println!(
                        "Field: {}, Size: {}, GlSize: {}",
                        field.name, field.size, field.gl_size
                    );
                    self.attribs.push(AttribPointer::new(
                        field.base,
                        field.gl_size,
                        0,
                        false,
                        stride,
                    ));
                    stride += field.size;
                }

                for attrib in self.attribs.iter_mut() {
                    attrib.stride = stride;
                }

                if stride == 0 {
                    panic!("No attribute pointer and no valid type");
                }
            }

            for (i, attrib) in self.attribs.into_iter().enumerate() {
                gl.enable_vertex_attrib_array(i as u32);
                gl.vertex_attrib_pointer_f32(
                    i as u32,
                    attrib.size,
                    attrib.ty as u32,
                    attrib.normalized,
                    attrib.stride,
                    attrib.offset,
                );
            }

            gl.bind_vertex_array(None);

            VertexArray {
                gl: gl.clone(),
                vao,
                array_buffer,
            }
        }
    }
}
