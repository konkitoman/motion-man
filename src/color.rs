use crate::gcx::vertex_array::{DataType, GLType};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl GLType for Color {
    fn base() -> DataType {
        DataType::F32
    }

    fn size() -> i32 {
        4
    }
}

impl Color {
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const ALPHA: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<i32> for Color {
    fn from(value: i32) -> Self {
        let a = (value & 255) as f32 / 255.;
        let r = (value >> 8 & 255) as f32 / 255.;
        let g = (value >> 16 & 255) as f32 / 255.;
        let b = (value >> 24 & 255) as f32 / 255.;

        Self { r, g, b, a }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: 1.0,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}
