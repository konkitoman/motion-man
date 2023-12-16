use std::rc::Rc;

use after_drop::AfterDropBoxed;
use GL::HasContext;

use super::{GCX, GL};

#[derive(Debug)]
pub struct Shader {
    gl: Rc<GL::Context>,
    pub program: GL::Program,
}

pub trait SetUniform: Sized {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation);
}

impl SetUniform for i32 {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_1_i32(Some(location), self) };
    }
}

impl SetUniform for (i32, i32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_2_i32(Some(location), self.0, self.1) };
    }
}

impl SetUniform for (i32, i32, i32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_3_i32(Some(location), self.0, self.1, self.2) };
    }
}

impl SetUniform for (i32, i32, i32, i32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_4_i32(Some(location), self.0, self.1, self.2, self.3) };
    }
}

impl SetUniform for u32 {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_1_u32(Some(location), self) };
    }
}

impl SetUniform for (u32, u32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_2_u32(Some(location), self.0, self.1) };
    }
}

impl SetUniform for (u32, u32, u32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_3_u32(Some(location), self.0, self.1, self.2) };
    }
}

impl SetUniform for (u32, u32, u32, u32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_4_u32(Some(location), self.0, self.1, self.2, self.3) };
    }
}

impl SetUniform for f32 {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_1_f32(Some(location), self) };
    }
}

impl SetUniform for (f32, f32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_2_f32(Some(location), self.0, self.1) };
    }
}

impl SetUniform for (f32, f32, f32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_3_f32(Some(location), self.0, self.1, self.2) };
    }
}

impl SetUniform for (f32, f32, f32, f32) {
    fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
        unsafe { gl.uniform_4_f32(Some(location), self.0, self.1, self.2, self.3) };
    }
}

// impl SetUniform for &[i32] {
//     fn set_uniform(self, gl: &GL::Context, location: &GL::NativeUniformLocation) {
//         unsafe { gl.uniform }
//     }
// }

impl Shader {
    pub fn set_uniform<T: SetUniform>(&self, name: &str, data: T) -> Result<(), T> {
        unsafe {
            let Some(location) = self.gl.get_uniform_location(self.program, name) else {
                return Err(data);
            };

            data.set_uniform(&self.gl, &location);
        }
        Ok(())
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}

pub struct ShaderVextex {
    src: String,
}
pub struct ShaderFragment {
    src: String,
}
pub struct ShaderCompute {
    src: String,
}

pub struct ShaderBuilder {
    vertex: Option<ShaderVextex>,
    fragment: Option<ShaderFragment>,
    compute: Option<ShaderCompute>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ShaderStage {
    Vertex = GL::VERTEX_SHADER,
    Fragment = GL::FRAGMENT_SHADER,
    Compute = GL::COMPUTE_SHADER,
    Geometry = GL::GEOMETRY_SHADER,
    TessControl = GL::TESS_CONTROL_SHADER,
    TessEveluation = GL::TESS_EVALUATION_SHADER,
}

#[derive(Debug)]
pub enum ShaderError {
    CreateShaderStage(ShaderStage, String),
    CreateShader(String),
    CompileError(ShaderStage, String),
    LinkError(String),
}

impl ShaderBuilder {
    pub fn new() -> Self {
        Self {
            vertex: None,
            fragment: None,
            compute: None,
        }
    }

    pub fn vertex(mut self, src: impl Into<String>) -> Self {
        self.vertex = Some(ShaderVextex { src: src.into() });
        self
    }

    pub fn fragment(mut self, src: impl Into<String>) -> Self {
        self.fragment = Some(ShaderFragment { src: src.into() });
        self
    }

    pub fn compute(mut self, src: impl Into<String>) -> Self {
        self.compute = Some(ShaderCompute { src: src.into() });
        self
    }

    pub fn build(self, gcx: &GCX) -> Result<Shader, ShaderError> {
        unsafe fn create_shader(
            gl: &GL::Context,
            ty: ShaderStage,
            src: &str,
        ) -> Result<GL::Shader, ShaderError> {
            let shader = gl
                .create_shader(ty as u32)
                .map_err(|err| ShaderError::CreateShaderStage(ty, err))?;
            gl.shader_source(shader, src);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let error = gl.get_shader_info_log(shader);
                return Err(ShaderError::CompileError(ty, error));
            }
            Ok(shader)
        }

        let program;

        unsafe {
            let gl = &gcx.gl;
            let mut defers = Vec::new();

            program = gl.create_program().map_err(ShaderError::CreateShader)?;

            if let Some(vertex_shader) = &self.vertex {
                let shader = create_shader(gl, ShaderStage::Vertex, &vertex_shader.src)?;
                gl.attach_shader(program, shader);

                defers.push(AfterDropBoxed::new(move || {
                    gl.delete_shader(shader);
                    println!("ShaderStage vertex deleted!");
                }));
            }

            if let Some(fragment_shader) = &self.fragment {
                let shader = create_shader(gl, ShaderStage::Fragment, &fragment_shader.src)?;
                gl.attach_shader(program, shader);

                defers.push(AfterDropBoxed::new(move || {
                    gl.delete_shader(shader);
                    println!("ShaderStage fragment deleted!");
                }));
            }

            if let Some(compute_shader) = &self.compute {
                let shader = create_shader(gl, ShaderStage::Compute, &compute_shader.src)?;
                gl.attach_shader(program, shader);

                defers.push(AfterDropBoxed::new(move || {
                    gl.delete_shader(shader);
                    println!("ShaderStage compute deleted!");
                }));
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let err = gl.get_program_info_log(program);
                return Err(ShaderError::LinkError(err));
            }

            Ok(Shader {
                program,
                gl: gl.clone(),
            })
        }
    }
}

impl Default for ShaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}
