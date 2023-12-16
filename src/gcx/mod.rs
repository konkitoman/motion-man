pub mod buffer;
pub mod shader;
pub mod texture;
pub mod vertex_array;

use std::rc::Rc;

pub use glow as GL;
use glow::HasContext;

use crate::color::Color;

use self::{
    buffer::{Buffer, BufferInner, BufferType, BufferUsage},
    shader::{Shader, ShaderBuilder},
    texture::{Format, InternalFormat, Texture, TextureInner, TextureTarget},
    vertex_array::{Fields, VertexArray, VertexArrayBuilder},
};

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DataType {
    U8 = GL::UNSIGNED_BYTE,
    U16 = GL::UNSIGNED_SHORT,
    U32 = GL::UNSIGNED_INT,
    I8 = GL::BYTE,
    I16 = GL::SHORT,
    I32 = GL::INT,
    F32 = GL::FLOAT,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BufferBit {
    COLOR = GL::COLOR_BUFFER_BIT,
    DEPTH = GL::DEPTH_BUFFER_BIT,
    STENCIL = GL::STENCIL_BUFFER_BIT,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PrimitiveType {
    Points = GL::POINTS,
    Lines = GL::LINES,
    LineLoop = GL::LINE_LOOP,
    LineStrip = GL::LINE_STRIP,
    Triangles = GL::TRIANGLES,
    TrianglesStrip = GL::TRIANGLE_STRIP,
    TrianglesFan = GL::TRIANGLE_FAN,
}

#[derive(Debug, Clone)]
pub struct GCX {
    pub gl: Rc<glow::Context>,
}

impl GCX {
    pub fn new(gl: Rc<GL::Context>) -> Self {
        Self { gl }
    }

    pub fn clear_color(&self, color: impl Into<Color>) {
        let color = color.into();
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
        }
    }

    pub fn clear(&self, buffer_bit: BufferBit) {
        unsafe { self.gl.clear(buffer_bit as u32) }
    }

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.gl.viewport(x, y, width, height) }
    }

    pub fn use_shader<O>(&self, shader: &Shader, run: impl FnOnce(GCXShaded) -> O) {
        unsafe {
            self.gl.use_program(Some(shader.program));
        }

        run(GCXShaded { gcx: self });

        unsafe {
            self.gl.use_program(None);
        }
    }

    pub fn create_shader(&self) -> ShaderBuilder {
        ShaderBuilder::default()
    }

    pub fn create_vertex_array<T: Fields>(&self, array_buffer: Buffer) -> VertexArrayBuilder<T> {
        VertexArrayBuilder {
            array_buffer,
            attribs: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn create_buffer<T: bytemuck::NoUninit + bytemuck::AnyBitPattern>(
        &self,
        ty: BufferType,
        data: &[T],
        usage: BufferUsage,
    ) -> Buffer {
        let gl = &self.gl;
        let buffer;
        unsafe {
            buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(ty as u32, Some(buffer));
            gl.buffer_data_u8_slice(ty as u32, bytemuck::cast_slice(data), usage.bits());
            gl.bind_buffer(ty as u32, None);
        }

        let gl = gl.clone();
        Buffer {
            inner: Rc::new(BufferInner { gl, buffer, ty }),
        }
    }

    pub fn create_texture<T: bytemuck::NoUninit>(
        &self,
        ty: texture::TextureType,
        target: TextureTarget,
        level: i32,
        internal_format: InternalFormat,
        width: i32,
        height: i32,
        format: Format,
        data_ty: DataType,
        data: &[T],
    ) -> Texture {
        let gl = &self.gl;
        let row;
        unsafe {
            row = gl.create_texture().unwrap();

            gl.bind_texture(target as u32, Some(row));
            gl.tex_image_2d(
                target as u32,
                level,
                internal_format as i32,
                width,
                height,
                0,
                format as u32,
                data_ty as u32,
                Some(bytemuck::cast_slice(data)),
            );
            gl.generate_mipmap(target as u32);
            gl.bind_texture(target as u32, None);
        }

        Texture {
            inner: Rc::new(TextureInner {
                gl: gl.clone(),
                row,
                format,
                internal_format,
                ty,
                width,
                height,
                target,
                data_ty,
            }),
        }
    }

    pub fn flush(&self) {
        unsafe {
            self.gl.flush();
        }
    }

    pub fn finish(&self) {
        unsafe { self.gl.finish() }
    }
}

pub struct GCXShaded<'a> {
    gcx: &'a GCX,
}

impl<'a> std::ops::Deref for GCXShaded<'a> {
    type Target = GCX;

    fn deref(&self) -> &Self::Target {
        self.gcx
    }
}

impl<'a> GCXShaded<'a> {
    pub fn use_vertex_array<O>(&self, va: &VertexArray, run: impl FnOnce(GCXFinal) -> O) {
        unsafe { self.gl.bind_vertex_array(Some(va.vao)) }
        run(GCXFinal { gcx: self });
        unsafe { self.gl.bind_vertex_array(None) }
    }
}

pub struct GCXFinal<'a> {
    gcx: &'a GCXShaded<'a>,
}

impl<'a> std::ops::Deref for GCXFinal<'a> {
    type Target = GCXShaded<'a>;

    fn deref(&self) -> &Self::Target {
        self.gcx
    }
}

impl<'a> GCXFinal<'a> {
    pub fn draw_arrays(&self, primitive: PrimitiveType, first: i32, count: i32) {
        unsafe { self.gl.draw_arrays(primitive as u32, first, count) }
    }

    pub fn draw_arrays_instanced(&self, primitive: PrimitiveType, first: i32, count: i32) {
        unsafe {
            self.gl
                .draw_arrays_instanced(primitive as u32, first, count, count - first)
        }
    }

    /// You should have GL_ELEMENT_ARRAY_BUFFER
    pub fn draw_elements(&self, primitive: PrimitiveType, count: i32) {
        unsafe {
            self.gl
                .draw_elements(primitive as u32, count, GL::UNSIGNED_INT, 0)
        }
    }
}
