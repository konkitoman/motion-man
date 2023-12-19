use std::any::{Any, TypeId};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    color::Color,
    element::ElementBuilder,
    gcx::{
        buffer::{BufferType, BufferUsage},
        shader::Shader,
        vertex_array::{Field, Fields, VertexArray},
        PrimitiveType, GCX,
    },
    node::Node,
    scene::SceneTask,
    signal::{create_signal, NSignal, Signal, SignalInner},
    SSAny,
};

#[derive(Debug)]
pub struct RectBuilder {
    pub(super) size: [f32; 2],
    pub(super) color: Color,
    pub(super) position: [f32; 2],
}

impl RectBuilder {
    pub fn new(size: [f32; 2], color: impl Into<Color>) -> Self {
        Self {
            size,
            color: color.into(),
            position: [0.; 2],
        }
    }

    pub fn with_position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }
}

pub struct Rect<'a> {
    scene: &'a SceneTask,

    pub position: Signal<'a, [f32; 2]>,
    pub size: Signal<'a, [f32; 2]>,
    pub color: Signal<'a, Color>,

    drop: Signal<'a, ()>,
    droped: bool,
}

impl<'a> Rect<'a> {
    pub async fn drop(mut self) {
        self.drop.set(()).await;
        self.scene.update().await;
        self.droped = true;
    }
}

impl<'a> Drop for Rect<'a> {
    fn drop(&mut self) {
        if self.droped {
            return;
        }

        eprintln!("You need to call drop on Rect when you are done with it!");
        std::process::abort();
    }
}

impl ElementBuilder for RectBuilder {
    type Element<'a> = Rect<'a>;

    fn node_id(&self) -> TypeId {
        core::any::TypeId::of::<RectNode>()
    }

    fn create_element_ref<'a>(&self, inner: Box<SSAny>, scene: &'a SceneTask) -> Self::Element<'a> {
        let (position, size, color, drop): (
            SignalInner<[f32; 2]>,
            SignalInner<[f32; 2]>,
            SignalInner<Color>,
            SignalInner<()>,
        ) = *inner.downcast().unwrap();

        Rect {
            scene,
            droped: false,
            position: Signal::new(position, scene, self.position),
            size: Signal::new(size, scene, self.size),
            color: Signal::new(color, scene, self.color),
            drop: Signal::new(drop, scene, ()),
        }
    }
}

pub struct NRect {
    va: VertexArray,
    builder: RectBuilder,
    inner: NRectInner,
}

pub struct NRectInner {
    drop: NSignal<()>,
    position: NSignal<[f32; 2]>,
    size: NSignal<[f32; 2]>,
    color: NSignal<Color>,
}

#[derive(Default)]
pub struct RectNode {
    pub(super) rects: Vec<NRect>,
    pub(super) shader: Option<Shader>,

    pending: Option<NRectInner>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    position: [f32; 2],
    color: Color,
}

impl Fields for RectVertex {
    fn fields() -> Vec<Field> {
        vec![
            Field::new::<[f32; 2]>("position"),
            Field::new::<Color>("color"),
        ]
    }
}

impl Node for RectNode {
    type ElementBuilder = RectBuilder;

    fn init(&mut self, gcx: &GCX) {
        let shader = gcx
            .create_shader()
            .vertex(
                r#"
                #version 320 es

                precision highp float;

                in vec2 pos;
                in vec4 color;

                out vec4 VertexColor;

                void main(){
                    gl_Position = vec4(pos, 0.0, 1.0);
                    VertexColor = color;
                }
            "#,
            )
            .fragment(
                r#"
                #version 320 es

                precision highp float;

                in vec4 VertexColor;
                out vec4 color;
                void main(){
                    color = VertexColor;
                }
                "#,
            )
            .build(gcx)
            .unwrap();

        self.shader.replace(shader);
    }

    fn init_element(&mut self, gcx: &GCX, builder: Self::ElementBuilder) {
        let buffer = gcx.create_buffer(
            BufferType::ArrayBuffer,
            &Self::build_mesh(&builder),
            BufferUsage::DRAW_STATIC,
        );
        let va = gcx.create_vertex_array::<RectVertex>(buffer).build(gcx);
        self.rects.push(NRect {
            va,
            builder,
            inner: self.pending.take().unwrap(),
        });
    }

    fn render(&self, gcx: &GCX) {
        let Some(shader) = &self.shader else { panic!() };
        gcx.use_shader(shader, |gcx| {
            for rect in self.rects.iter() {
                gcx.use_vertex_array(&rect.va, |gcx| {
                    gcx.draw_arrays(PrimitiveType::TrianglesFan, 0, 4);
                });
            }
        });
    }

    fn create_element(&mut self) -> Box<SSAny> {
        let (nposition, position) = create_signal();
        let (nsize, size) = create_signal();
        let (ncolor, color) = create_signal();
        let (ndrop, drop) = create_signal();

        self.pending = Some(NRectInner {
            drop,
            position,
            size,
            color,
        });

        Box::new((nposition, nsize, ncolor, ndrop))
    }

    fn update(&mut self) {
        self.rects.retain_mut(|rect| {
            let mut rebuild = false;
            if let Some(position) = rect.inner.position.get() {
                rect.builder.position = position;
                rebuild = true;
            }
            if let Some(size) = rect.inner.size.get() {
                rect.builder.size = size;
                rebuild = true;
            }
            if let Some(color) = rect.inner.color.get() {
                rect.builder.color = color;
                rebuild = true;
            }

            if let Some(_) = rect.inner.drop.get() {
                return false;
            }

            if rebuild {
                rect.va
                    .array_buffer
                    .update(0, &RectNode::build_mesh(&rect.builder));
            }
            true
        });
    }
}

impl RectNode {
    fn build_mesh(builder: &RectBuilder) -> [RectVertex; 4] {
        let color = builder.color;
        let size = builder.size;
        let position = builder.position;
        [
            RectVertex {
                position: [-size[0] + position[0], -size[1] + position[1]],
                color,
            },
            RectVertex {
                position: [-size[0] + position[0], size[1] + position[1]],
                color,
            },
            RectVertex {
                position: [size[0] + position[0], size[1] + position[1]],
                color,
            },
            RectVertex {
                position: [size[0] + position[0], -size[1] + position[1]],
                color,
            },
        ]
    }
}
