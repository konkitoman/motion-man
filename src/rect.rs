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

pub struct RectRef {
    sender: Sender<ElementMessage>,
}

impl RectRef {
    pub fn set_size(&mut self, size: [f32; 2]) {
        self.sender
            .try_send(ElementMessage::Set(0, Box::new(size)))
            .unwrap();
    }

    pub fn set_color(&mut self, color: impl Into<Color>) {
        if self
            .sender
            .try_send(ElementMessage::Set(1, Box::new(color.into())))
            .is_err()
        {
            eprintln!("You can only set one element property per update!");
            panic!();
        }
    }

    pub fn set_position(&mut self, position: [f32; 2]) {
        self.sender
            .try_send(ElementMessage::Set(2, Box::new(position)))
            .unwrap();
    }
}

impl ElementBuilder for RectBuilder {
    type ElementRef = RectRef;

    fn node_id(&self) -> TypeId {
        core::any::TypeId::of::<RectNode>()
    }

    fn create_element_ref(&self, sender: Sender<ElementMessage>) -> Self::ElementRef {
        RectRef { sender }
    }
}

#[derive(Default, Debug)]
pub struct RectNode {
    pub(super) rects: Vec<(VertexArray, RectBuilder)>,
    pub(super) receivers: Vec<Receiver<ElementMessage>>,
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
            &Self::build_mesh(builder.size, builder.position, builder.color),
            BufferUsage::DRAW_STATIC,
        );
        let vao = gcx.create_vertex_array::<RectVertex>(buffer).build(gcx);
        self.rects.push((vao, builder));
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
        self.receivers.push(recv);
        send
    }

    fn update(&mut self) {
        for (i, recv) in self.receivers.iter_mut().enumerate() {
            if let Ok(msg) = recv.try_recv() {
                match msg {
                    ElementMessage::Set(0, data) => {
                        let Ok(size) = data.downcast::<[f32; 2]>() else {
                            continue;
                        };
                        let color = self.rects[i].1.color;
                        let position = self.rects[i].1.position;
                        self.rects[i]
                            .0
                            .array_buffer
                            .update(0, &Self::build_mesh(*size, position, color));
                        self.rects[i].1.size = *size;
                    }

                    ElementMessage::Set(1, data) => {
                        let Ok(color) = data.downcast::<Color>() else {
                            continue;
                        };
                        let size = self.rects[i].1.size;
                        let position = self.rects[i].1.position;
                        self.rects[i]
                            .0
                            .array_buffer
                            .update(0, &Self::build_mesh(size, position, *color));
                        self.rects[i].1.color = *color;
                    }

                    ElementMessage::Set(2, data) => {
                        let Ok(position) = data.downcast::<[f32; 2]>() else {
                            continue;
                        };
                        let size = self.rects[i].1.size;
                        let color = self.rects[i].1.color;
                        self.rects[i]
                            .0
                            .array_buffer
                            .update(0, &Self::build_mesh(size, *position, color));
                        self.rects[i].1.position = *position;
                    }

                    _ => continue,
                }
            }
        }
    }
}

impl RectNode {
    fn build_mesh(size: [f32; 2], position: [f32; 2], color: Color) -> [RectVertex; 4] {
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
