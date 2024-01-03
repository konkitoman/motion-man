use crate::{
    color::Color,
    gcx::{
        buffer::{BufferType, BufferUsage},
        shader::Shader,
        vertex_array::{Field, Fields, VertexArray},
        PrimitiveType, GCX,
    },
    node::{NodeBuilder, NodeManager},
    scene::SceneTask,
    signal::{create_signal, NSignal, RawSignal, Signal},
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
    dropped: bool,
}

impl<'a> Rect<'a> {
    pub async fn drop(mut self) {
        self.drop.set(()).await;
        self.scene.update().await;
        self.dropped = true;
    }
}

impl<'a> Drop for Rect<'a> {
    fn drop(&mut self) {
        if self.dropped {
            return;
        }

        eprintln!("You need to call drop on Rect when you are done with it!");
        std::process::abort();
    }
}

impl NodeBuilder for RectBuilder {
    type Node<'a> = Rect<'a>;
    type NodeManager = RectNodeManager;

    fn create_node_ref<'a>(&self, raw: RawRect, scene: &'a SceneTask) -> Self::Node<'a> {
        Rect {
            scene,
            dropped: false,
            position: Signal::new(raw.position, scene, self.position),
            size: Signal::new(raw.size, scene, self.size),
            color: Signal::new(raw.color, scene, self.color),
            drop: Signal::new(raw.drop, scene, ()),
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

pub struct RawRect {
    drop: RawSignal<()>,
    position: RawSignal<[f32; 2]>,
    size: RawSignal<[f32; 2]>,
    color: RawSignal<Color>,
}

#[derive(Default)]
pub struct RectNodeManager {
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

impl NodeManager for RectNodeManager {
    type NodeBuilder = RectBuilder;
    type RawNode = RawRect;

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

    fn init_node(&mut self, gcx: &GCX, builder: Self::NodeBuilder) {
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

    fn render(&mut self, gcx: &GCX) {
        let Some(shader) = &self.shader else { panic!() };
        gcx.use_shader(shader, |gcx| {
            for rect in self.rects.iter() {
                gcx.use_vertex_array(&rect.va, |gcx| {
                    gcx.draw_arrays(PrimitiveType::TrianglesFan, 0, 4);
                });
            }
        });
    }

    fn create_node(&mut self) -> RawRect {
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

        RawRect {
            drop: ndrop,
            position: nposition,
            size: nsize,
            color: ncolor,
        }
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

            if rect.inner.drop.get().is_some() {
                return false;
            }

            if rebuild {
                rect.va
                    .array_buffer
                    .update(0, &RectNodeManager::build_mesh(&rect.builder));
            }
            true
        });
    }
}

impl RectNodeManager {
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
