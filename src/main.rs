use std::{
    any::{Any, TypeId},
    error::Error,
    future::Future,
    num::NonZeroU32,
    ops::RangeInclusive,
    pin::{pin, Pin},
    rc::Rc,
    sync::Arc,
    task::Poll::{self, Ready},
    time::{Duration, Instant},
};

use after_drop::AfterDropBoxed;
use glow as GL;
use glow::HasContext;
use glutin::{
    config::{Config, ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::HasRawWindowHandle;
use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
    task::JoinHandle,
};
use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};

type ORecv<T> = tokio::sync::oneshot::Receiver<T>;
type OSend<T> = tokio::sync::oneshot::Sender<T>;
use tokio::sync::oneshot::channel as ochannel;

#[derive(Debug)]
pub enum EngineMessage {
    CreateRef(TypeId, OSend<Sender<ElementMessage>>),
    CreateElement(TypeId, Box<dyn Any + Send + Sync + 'static>),
    WaitNextFrame(OSend<()>),
    Submit,
}

pub enum SceneMessage {
    NextFrame,
    Resumed,
}

pub struct EngineSender {
    id: usize,
    sender: Sender<(usize, EngineMessage)>,
}

impl EngineSender {
    pub async fn send(&self, msg: EngineMessage) {
        self.sender.send((self.id, msg)).await.unwrap();
    }
}

pub struct SceneTask {
    sender: EngineSender,

    info: Arc<RwLock<Info>>,
}

pub struct Info {
    pub delta: f64,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl Info {
    pub fn fps(&self) -> usize {
        (1. / self.delta).round() as usize
    }
}

pub struct Tween<'a> {
    range: RangeInclusive<f32>,
    time: f32,
    runner: Box<dyn FnMut(f32) + Send + Sync + 'a>,
    x: f32,
}

impl<'a> Tween<'a> {
    pub fn new(
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + Send + Sync + 'a,
    ) -> Self {
        Self {
            x: *range.start(),
            range,
            time,
            runner: Box::new(runner),
        }
    }
}

pub enum TweenBuilderStage<'a> {
    Init {
        task: &'a mut SceneTask,
        tweens: Vec<Tween<'a>>,
    },
    Running(Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>),
}

pub struct TweenBuilder<'a> {
    stage: Option<TweenBuilderStage<'a>>,
}

impl<'a> TweenBuilder<'a> {
    pub fn new(task: &'a mut SceneTask, tween: Tween<'a>) -> Self {
        Self {
            stage: Some(TweenBuilderStage::Init {
                task,
                tweens: vec![tween],
            }),
        }
    }

    pub fn tween(
        mut self,
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + Sync + Send + 'a,
    ) -> Self {
        if let TweenBuilderStage::Init { tweens, .. } = self.stage.as_mut().unwrap() {
            tweens.push(Tween::new(range, time, runner));
        }
        self
    }
}

impl<'a> Future for TweenBuilder<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let task = self.stage.take().unwrap();

        match task {
            TweenBuilderStage::Init { task, mut tweens } => {
                let future = Box::pin(async move {
                    let delta = task.info(|i| i.delta).await as f32;
                    loop {
                        tweens.retain_mut(|tween| {
                            let start = *tween.range.start();
                            let end = *tween.range.end();

                            let inverse = start > end;
                            if inverse {
                                tween.x -= (delta / tween.time) * (start - end);
                                (tween.runner)(tween.x);

                                tween.x >= end
                            } else {
                                tween.x += (delta / tween.time) * (end - start);
                                (tween.runner)(tween.x);

                                tween.x <= end
                            }
                        });
                        task.submit().await;
                        task.wait(1).await;

                        if tweens.is_empty() {
                            break;
                        }
                    }
                });

                self.stage.replace(TweenBuilderStage::Running(future));
            }
            TweenBuilderStage::Running(mut run) => {
                let res = pin!(&mut run).poll(cx);
                self.stage.replace(TweenBuilderStage::Running(run));
                return res;
            }
        }

        if let TweenBuilderStage::Running(mut run) = self.stage.take().unwrap() {
            let res = pin!(&mut run).poll(cx);
            self.stage.replace(TweenBuilderStage::Running(run));
            res
        } else {
            Ready(())
        }
    }
}

impl SceneTask {
    pub async fn wait(&mut self, frames: usize) {
        for _ in 0..frames {
            let (send, recv) = ochannel();
            let _ = self.sender.send(EngineMessage::WaitNextFrame(send)).await;
            recv.await.unwrap();
        }
    }

    pub async fn info<O>(&self, reader: impl Fn(&Info) -> O) -> O {
        let info = self.info.read().await;
        reader(&info)
    }

    pub async fn spawn_element<T: ElementBuilder + 'static>(&self, builder: T) -> T::ElementRef {
        let (send, recv) = ochannel();
        self.sender
            .send(EngineMessage::CreateRef(builder.node_id(), send))
            .await;

        let element_ref = builder.create_element_ref(recv.await.unwrap());

        self.sender
            .send(EngineMessage::CreateElement(
                builder.node_id(),
                Box::new(builder),
            ))
            .await;

        element_ref
    }

    pub async fn submit(&mut self) {
        self.sender.send(EngineMessage::Submit).await;
    }

    pub fn tween<'a>(
        &'a mut self,
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + 'a + Sync + Send,
    ) -> TweenBuilder<'a> {
        TweenBuilder::new(self, Tween::new(range, time, runner))
    }
}

pub struct EngineScene {
    id: usize,
    handler: JoinHandle<()>,
}

pub struct Engine {
    scenes: Vec<EngineScene>,

    counter: usize,
    engine_sender: Sender<(usize, EngineMessage)>,
    receiver: Receiver<(usize, EngineMessage)>,

    waiting: Vec<OSend<()>>,

    info: Arc<RwLock<Info>>,

    nodes: Vec<Box<dyn AbstractNode>>,
}

pub enum ElementMessage {
    Set(u32, Box<dyn Any + Send + Sync + 'static>),
}

pub trait ElementBuilder: core::fmt::Debug + Send + Sync {
    type ElementRef;

    fn node_id(&self) -> TypeId;

    fn create_element_ref(&self, sender: Sender<ElementMessage>) -> Self::ElementRef;
}

pub trait AbstractElementBuilder: core::fmt::Debug + Send + Sync {
    fn node_id(&self) -> TypeId;
}

pub trait AbstractNode: core::fmt::Debug {
    fn init(&mut self, gcx: &GCX);
    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>);

    fn create_ref(&mut self) -> Sender<ElementMessage>;

    fn update(&mut self);
    fn render(&self, gcx: &GCX);

    fn ty_id(&self) -> TypeId;
}

impl<T: Node + core::fmt::Debug + 'static> AbstractNode for T {
    fn init(&mut self, gcx: &GCX) {
        self.init(gcx)
    }

    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>) {
        let builder = Box::<dyn Any + Send + Sync>::downcast::<T::ElementBuilder>(builder).unwrap();
        self.init_element(gcx, *builder);
    }

    fn create_ref(&mut self) -> Sender<ElementMessage> {
        self.create_ref()
    }

    fn update(&mut self) {
        self.update();
    }

    fn render(&self, gcx: &GCX) {
        self.render(gcx);
    }

    fn ty_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub trait Node {
    type ElementBuilder;

    fn init(&mut self, gcx: &GCX);

    fn init_element(&mut self, gcx: &GCX, builder: Self::ElementBuilder);
    fn create_ref(&mut self) -> Sender<ElementMessage>;

    fn update(&mut self);
    fn render(&self, gcx: &GCX);
}

#[derive(Debug)]
pub struct RectBuilder {
    size: [f32; 2],
    color: Color,
    position: [f32; 2],
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
    rects: Vec<(VertexArray, RectBuilder)>,
    receivers: Vec<Receiver<ElementMessage>>,
    shader: Option<Shader>,
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

impl Engine {
    pub fn new(fps: f64, width: NonZeroU32, height: NonZeroU32) -> Self {
        let info = Info {
            delta: 1. / fps,
            width,
            height,
        };

        let (engine_sender, receiver) = channel(8);

        Self {
            scenes: Vec::default(),
            info: Arc::new(RwLock::new(info)),
            nodes: Vec::default(),
            counter: 0,
            engine_sender,
            receiver,
            waiting: Vec::default(),
        }
    }

    pub fn create_scene(
        &mut self,
        scene_run: impl Fn(SceneTask) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    ) {
        let id = self.counter;
        self.counter += 1;

        let scene = SceneTask {
            sender: EngineSender {
                id,
                sender: self.engine_sender.clone(),
            },

            info: self.info.clone(),
        };

        let engine_scene = EngineScene {
            id,
            handler: spawn(scene_run(scene)),
        };

        self.scenes.push(engine_scene);
    }

    pub fn register_node<T: AbstractNode + Default + 'static>(&mut self) {
        let node = T::default();
        self.nodes.push(Box::new(node));
    }

    pub fn render(&self, gcx: &GCX) {
        for node in self.nodes.iter() {
            node.render(gcx);
        }
    }

    pub fn finished(&self) -> bool {
        self.scenes.is_empty()
    }

    pub fn init(&mut self, gcx: &GCX) {
        for node in self.nodes.iter_mut() {
            node.init(gcx);
        }
    }

    pub async fn run(&mut self, gcx: &GCX) {
        for waiting in self.waiting.drain(..) {
            waiting.send(()).unwrap();
        }

        loop {
            tokio::task::yield_now().await;
            if let Ok((from, msg)) = self.receiver.try_recv() {
                println!("From: {from}, {msg:?}");
                match msg {
                    EngineMessage::WaitNextFrame(send) => {
                        self.waiting.push(send);
                    }
                    EngineMessage::CreateElement(type_id, builder) => {
                        for node in self.nodes.iter_mut() {
                            if node.ty_id() == type_id {
                                node.init_element(gcx, builder);
                                break;
                            }
                        }
                    }
                    EngineMessage::CreateRef(type_id, send) => {
                        for node in self.nodes.iter_mut() {
                            if node.ty_id() == type_id {
                                let s = node.create_ref();
                                send.send(s).unwrap();
                                break;
                            }
                        }
                    }
                    EngineMessage::Submit => {
                        for node in self.nodes.iter_mut() {
                            node.update();
                        }
                    }
                }
            }

            self.scenes
                .retain(|EngineScene { handler, .. }| !handler.is_finished());

            if self.scenes.len() <= self.waiting.len() {
                break;
            }
        }

        println!("Scenes in processing: {}", self.scenes.len());
    }
}

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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BufferBit {
    COLOR = GL::COLOR_BUFFER_BIT,
    DEPTH = GL::DEPTH_BUFFER_BIT,
    STENCIL = GL::STENCIL_BUFFER_BIT,
}

#[derive(Debug)]
pub struct Shader {
    gl: Rc<GL::Context>,
    program: GL::Program,
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

#[derive(Debug)]
pub struct VertexArray {
    gl: Rc<glow::Context>,
    vao: GL::VertexArray,

    array_buffer: Buffer,
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
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
    array_buffer: Buffer,

    attribs: Vec<AttribPointer>,
    _marker: core::marker::PhantomData<T>,
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
    gl: Rc<glow::Context>,
    buffer: GL::Buffer,
    ty: BufferType,
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

    fn bind(&self) {
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
    inner: Rc<BufferInner>,
}

#[derive(Debug, Clone)]
pub struct GCX {
    gl: Rc<glow::Context>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct BufferUsage: u32 {
        const DRAW_STREAM = GL::STREAM_DRAW;
        const DRAW_STATIC = GL::STATIC_DRAW;
        const DRAW_DYNAMIC = GL::DYNAMIC_DRAW;
    }

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

fn make_context() -> Result<
    (
        EventLoop<()>,
        winit::window::Window,
        Config,
        PossiblyCurrentContext,
        Surface<WindowSurface>,
        GL::Context,
    ),
    Box<dyn Error>,
> {
    let event_loop = EventLoopBuilder::new().build().unwrap();

    let (_, config) = glutin_winit::DisplayBuilder::new()
        .build(&event_loop, ConfigTemplateBuilder::new(), |config| {
            let configs = config.collect::<Vec<_>>();
            let mut config = configs.first().unwrap().clone();
            let mut index = 0;
            println!("Configs:");
            for (i, new_config) in configs.into_iter().enumerate() {
                config = new_config;
                let color_buffer_type = config.color_buffer_type();
                let float_pixels = config.float_pixels();
                let alpha_size = config.alpha_size();
                let depth_size = config.depth_size();
                let stencil_size = config.stencil_size();
                let num_samples = config.num_samples();
                let srgb_capable = config.srgb_capable();
                let supports_transparency = config.supports_transparency();
                let hardware_accelerated = config.hardware_accelerated();
                let config_surface_types = config.config_surface_types();
                let api = config.api();
                println!("{i}:");
                println!("  ColorBufferType: {color_buffer_type:?}");
                println!("  FloatPixels: {float_pixels}");
                println!("  AlphaSize: {alpha_size}");
                println!("  DepthSize: {depth_size}");
                println!("  StencilSize: {stencil_size}");
                println!("  NumSamples: {num_samples}");
                println!("  SrgbCapable: {srgb_capable}");
                println!("  SupportsTransparency: {supports_transparency:?}");
                println!("  HardwareAccelerated: {hardware_accelerated}");
                println!("  SurfaceTypes: {config_surface_types:?}");
                println!("  Api: {api:?}");
                match config {
                    Config::Egl(_) => println!("  Backend: EGL"),
                    Config::Glx(_) => println!("  Backend: Glx"),
                    _ => {
                        println!("  Backend: Unknown")
                    }
                }
                index = i;
            }
            println!("Was selected: {index}");
            config
        })
        .unwrap();

    let display = config.display();
    let context = unsafe {
        display
            .create_context(&config, &ContextAttributesBuilder::new().build(None))
            .unwrap()
    };

    let window = glutin_winit::finalize_window(&event_loop, WindowBuilder::new(), &config).unwrap();

    let surface = unsafe {
        display
            .create_window_surface(
                &config,
                &SurfaceAttributesBuilder::<WindowSurface>::new().build(
                    window.raw_window_handle(),
                    500.try_into()?,
                    500.try_into()?,
                ),
            )
            .unwrap()
    };

    let context = context.make_current(&surface).unwrap();

    let mut gl =
        unsafe { GL::Context::from_loader_function_cstr(|c_str| display.get_proc_address(c_str)) };

    unsafe {
        gl.debug_message_callback(|source, ty, severity, d, detalis| {
            let source = match source {
                GL::DEBUG_SOURCE_API => "Api".into(),
                GL::DEBUG_SOURCE_APPLICATION => "Application".into(),
                GL::DEBUG_SOURCE_OTHER => "Other".into(),
                GL::DEBUG_SOURCE_SHADER_COMPILER => "ShaderCompiler".into(),
                GL::DEBUG_SOURCE_THIRD_PARTY => "ThirdParty".into(),
                GL::DEBUG_SOURCE_WINDOW_SYSTEM => "WindowSystem".into(),
                _ => {
                    format!("{source:X}")
                }
            };
            let ty = match ty {
                GL::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DeprecatedBehaviour".into(),
                GL::DEBUG_TYPE_ERROR => "Error".into(),
                GL::DEBUG_TYPE_MARKER => "Marker".into(),
                GL::DEBUG_TYPE_OTHER => "Other".into(),
                GL::DEBUG_TYPE_PERFORMANCE => "Parformance".into(),
                GL::DEBUG_TYPE_POP_GROUP => "PopGroup".into(),
                GL::DEBUG_TYPE_PORTABILITY => "Portability".into(),
                GL::DEBUG_TYPE_PUSH_GROUP => "PushGroup".into(),
                GL::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undifined Behaviour".into(),
                _ => format!("{ty:X}"),
            };
            let severity = match severity {
                GL::DEBUG_SEVERITY_HIGH => "HIGH".into(),
                GL::DEBUG_SEVERITY_LOW => "LOW".into(),
                GL::DEBUG_SEVERITY_MEDIUM => "MEDI".into(),
                GL::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION".into(),
                GL::INVALID_OPERATION => "INVALID_OPERATION".into(),
                _ => format!("{severity:X}"),
            };
            println!("{source} {ty} {severity} {d}: {detalis}");
        });
        gl.enable(GL::DEBUG_OUTPUT)
    }
    Ok((event_loop, window, config, context, surface, gl))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (event_loop, window, config, context, surface, gl) = make_context()?;

    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let _enter = rt.enter();

    let mut engine = Engine::new(144., 1920.try_into()?, 1080.try_into()?);

    engine.register_node::<RectNode>();

    engine.create_scene(|mut scene| {
        Box::pin(async move {
            scene
                .info(|info| {
                    println!("FPS: {}", info.fps());
                    println!("Width: {}", info.width);
                    println!("Height: {}", info.height);
                })
                .await;

            let mut rect = scene
                .spawn_element(RectBuilder::new([1., 1.], 0xffff))
                .await;

            scene.wait(scene.info(|i| i.fps()).await).await;

            rect.set_color(Color::GREEN);

            scene.submit().await;
            scene.wait(1).await;

            let mut rect2 = scene
                .spawn_element(
                    RectBuilder::new([0.5, 0.5], Color::BLUE).with_position([-0.5, -0.5]),
                )
                .await;

            scene.wait(1).await;

            scene
                .tween(-0.5..=0.5, 1.0, |x| rect2.set_position([x, -0.5]))
                .await;

            scene
                .tween(-0.5..=0.5, 1.0, |y| rect2.set_position([0.5, y]))
                .await;

            scene
                .tween(0.5..=-0.5, 1.0, |x| rect2.set_position([x, 0.5]))
                .await;

            scene
                .tween(0.5..=-0.5, 1.0, |y| rect2.set_position([-0.5, y]))
                .await;

            scene
                .tween(-0.5..=0.0, 1.0, |i| rect2.set_position([i, i]))
                .await;
        })
    });

    let gcx = GCX::new(Rc::new(gl));

    let width = engine.info.try_read().unwrap().width;
    let height = engine.info.try_read().unwrap().height;
    _ = window.request_inner_size(LogicalSize::new(width.get(), height.get()));
    surface.resize(&context, width, height);
    gcx.viewport(0, 0, width.get() as i32, height.get() as i32);

    engine.init(&gcx);

    loop {
        let instant = Instant::now();
        gcx.clear_color(0xff);
        gcx.clear(BufferBit::COLOR);

        rt.block_on(engine.run(&gcx));
        engine.render(&gcx);
        surface.swap_buffers(&context).unwrap();

        if let Some(remaining) = Duration::from_secs_f64(engine.info.blocking_read().delta)
            .checked_sub(instant.elapsed())
        {
            std::thread::sleep(remaining);
        } else {
            eprintln!(
                "Cannot keep up!!! late with: {}s",
                instant.elapsed().as_secs_f64()
            );
        }

        if engine.finished() {
            break;
        }
    }

    Ok(())
}
