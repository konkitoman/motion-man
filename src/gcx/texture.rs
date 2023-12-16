use crate::gcx::*;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum InternalFormat {
    R8 = GL::R8,
    R8S = GL::R8_SNORM,
    R16F = GL::R16F,
    R32F = GL::R32F,
    R8UI = GL::R8UI,
    R16UI = GL::R16UI,
    R32UI = GL::R32UI,
    R8I = GL::R8I,
    R16I = GL::R16I,
    R32I = GL::R32I,
    RG8 = GL::RG8,
    RG8S = GL::RG8_SNORM,
    RG16F = GL::RG16F,
    RG32F = GL::RG32F,
    RG8UI = GL::RG8UI,
    RG16UI = GL::RG16UI,
    RG32UI = GL::RG32UI,
    RG8I = GL::RG8I,
    RG16I = GL::RG16I,
    RG32I = GL::RG32I,
    RGB8 = GL::RGB8,
    RGB8S = GL::RGB8_SNORM,
    RGB16F = GL::RGB16F,
    RGB32F = GL::RGB32F,
    RGB8UI = GL::RGB8UI,
    RGB16UI = GL::RGB16UI,
    RGB32UI = GL::RGB32UI,
    RGB8I = GL::RGB8I,
    RGB16I = GL::RGB16I,
    RGB32I = GL::RGB32I,
    RGBA8 = GL::RGBA8,
    RGBA8S = GL::RGBA8_SNORM,
    RGBA16F = GL::RGBA16F,
    RGBA32F = GL::RGBA32F,
    RGBA8UI = GL::RGBA8UI,
    RGBA16UI = GL::RGBA16UI,
    RGBA32UI = GL::RGBA32UI,
    RGBA8I = GL::RGBA8I,
    RGBA16I = GL::RGBA16I,
    RGBA32I = GL::RGBA32I,

    DepthComponent16 = GL::DEPTH_COMPONENT16,
    DepthComponent24 = GL::DEPTH_COMPONENT24,
    DepthComponent32 = GL::DEPTH_COMPONENT32,
    DepthComponent32F = GL::DEPTH_COMPONENT32F,

    Depth24Stencil8 = GL::DEPTH24_STENCIL8,
    Depth32FStencil8 = GL::DEPTH32F_STENCIL8,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum Format {
    Red = GL::RED,
    RedInt = GL::RED_INTEGER,
    RG = GL::RG,
    RGInt = GL::RG_INTEGER,
    RGB = GL::RGB,
    RGBInt = GL::RGB_INTEGER,
    RGBA = GL::RGBA,
    RGBAInt = GL::RGBA_INTEGER,
    DepthComponent = GL::DEPTH_COMPONENT,
    DepthStencil = GL::DEPTH_STENCIL,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    Tex2D,
    Tex3D,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureTarget {
    Tex2D = GL::TEXTURE_2D,
    Tex3D = GL::TEXTURE_3D,
    Tex2DArray = GL::TEXTURE_2D_ARRAY,

    CubeMapPositiveX = GL::TEXTURE_CUBE_MAP_POSITIVE_X,
    CubeMapPositiveY = GL::TEXTURE_CUBE_MAP_POSITIVE_Y,
    CubeMapPositiveZ = GL::TEXTURE_CUBE_MAP_POSITIVE_Z,

    CubeMapNegativeX = GL::TEXTURE_CUBE_MAP_NEGATIVE_X,
    CubeMapNegativeY = GL::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    CubeMapNegativeZ = GL::TEXTURE_CUBE_MAP_NEGATIVE_Z,
}

#[derive(Clone)]
pub struct Texture {
    pub(super) inner: Rc<TextureInner>,
}

impl core::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Texture {
    pub fn internal_format(&self) -> InternalFormat {
        self.inner.internal_format
    }

    pub fn format(&self) -> Format {
        self.inner.format
    }

    pub fn target(&self) -> TextureTarget {
        self.inner.target
    }

    pub fn width(&self) -> i32 {
        self.inner.width
    }

    pub fn height(&self) -> i32 {
        self.inner.height
    }

    pub fn ty(&self) -> TextureType {
        self.inner.ty
    }

    pub fn data_ty(&self) -> DataType {
        self.inner.data_ty
    }

    pub fn update<T: bytemuck::NoUninit>(&self, level: i32, data: &[T]) {
        let row = self.inner.row;
        let gl = &self.inner.gl;
        let target = self.target();
        let internal_format = self.internal_format();
        let width = self.width();
        let height = self.height();
        let format = self.format();
        let data_ty = self.data_ty();

        unsafe {
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
    }

    pub fn activate(&self, unit: u32) {
        unsafe {
            self.inner.gl.active_texture(GL::TEXTURE0 + unit);
            self.inner
                .gl
                .bind_texture(self.inner.target as u32, Some(self.inner.row))
        }
    }
}

#[derive(Debug)]
pub(super) struct TextureInner {
    pub(super) gl: Rc<GL::Context>,
    pub(super) row: GL::Texture,
    pub(super) format: Format,
    pub(super) internal_format: InternalFormat,
    pub(super) ty: TextureType,
    pub(super) data_ty: DataType,
    pub(super) target: TextureTarget,
    pub(super) width: i32,
    pub(super) height: i32,
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        unsafe { self.gl.delete_texture(self.row) }
    }
}
