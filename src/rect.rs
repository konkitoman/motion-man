use std::any::TypeId;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    color::Color,
    element::{ElementBuilder, ElementMessage},
    gcx::{
        buffer::{BufferType, BufferUsage},
        shader::Shader,
        vertex_array::{Field, Fields, VertexArray},
        PrimitiveType, GCX,
    },
    node::Node,
    scene::SceneTask,
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
    sender: Sender<ElementMessage>,
    scene: &'a SceneTask,
    droped: bool,
}

impl<'a> Rect<'a> {
    pub fn set_size(&self, size: [f32; 2]) {
        self.sender
            .try_send(ElementMessage::Set(0, Box::new(size)))
            .unwrap();
    }

    pub fn set_color(&self, color: impl Into<Color>) {
        if self
            .sender
            .try_send(ElementMessage::Set(1, Box::new(color.into())))
            .is_err()
        {
            eprintln!("You can only set one element property per update!");
            panic!();
        }
    }

    pub fn set_position(&self, position: [f32; 2]) {
        self.sender
            .try_send(ElementMessage::Set(2, Box::new(position)))
            .unwrap();
    }

    pub async fn drop(mut self) {
        self.scene.submit().await;
        self.sender
            .send(ElementMessage::Set(21, Box::new(())))
            .await
            .unwrap();
        self.scene.submit().await;
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
    type ElementRef<'a> = Rect<'a>;

    fn node_id(&self) -> TypeId {
        core::any::TypeId::of::<RectNode>()
    }

    fn create_element_ref<'a>(
        &self,
        sender: Sender<ElementMessage>,
        scene: &'a SceneTask,
    ) -> Self::ElementRef<'a> {
        Rect {
            sender,
            scene,
            droped: false,
        }
    }
}

#[derive(Default, Debug)]
pub struct RectNode {
    pub(super) rects: Vec<(VertexArray, RectBuilder, u64)>,
    pub(super) receivers: Vec<(u64, Receiver<ElementMessage>)>,
    counter: u64,
    pub(super) shader: Option<Shader>,
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
        let vao = gcx.create_vertex_array::<RectVertex>(buffer).build(gcx);
        self.rects.push((vao, builder, self.counter));
        self.counter += 1;
    }

    fn render(&self, gcx: &GCX) {
        let Some(shader) = &self.shader else { panic!() };
        gcx.use_shader(shader, |gcx| {
            for rect in self.rects.iter() {
                gcx.use_vertex_array(&rect.0, |gcx| {
                    gcx.draw_arrays(PrimitiveType::TrianglesFan, 0, 4);
                });
            }
        });
    }

    fn create_ref(&mut self) -> Sender<ElementMessage> {
        let (send, recv) = channel(1);
        self.receivers.push((self.counter, recv));
        send
    }

    fn update(&mut self) {
        self.receivers.retain_mut(|(id, recv)| {
            if let Ok(msg) = recv.try_recv() {
                match msg {
                    ElementMessage::Set(0, data) => {
                        let size = data.downcast::<[f32; 2]>().unwrap();
                        for rect in self.rects.iter_mut() {
                            if rect.2 == *id {
                                rect.1.size = *size;
                                rect.0.array_buffer.update(0, &Self::build_mesh(&rect.1));
                                break;
                            }
                        }
                    }

                    ElementMessage::Set(1, data) => {
                        let color = data.downcast::<Color>().unwrap();
                        for rect in self.rects.iter_mut() {
                            if rect.2 == *id {
                                rect.1.color = *color;
                                rect.0.array_buffer.update(0, &Self::build_mesh(&rect.1));
                                break;
                            }
                        }
                    }

                    ElementMessage::Set(2, data) => {
                        let position = data.downcast::<[f32; 2]>().unwrap();
                        for rect in self.rects.iter_mut() {
                            if rect.2 == *id {
                                rect.1.position = *position;
                                rect.0.array_buffer.update(0, &Self::build_mesh(&rect.1));
                                break;
                            }
                        }
                    }
                    ElementMessage::Set(21, _) => {
                        self.rects.retain(|rect| rect.2 != *id);
                        return false;
                    }

                    _ => {}
                }
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
