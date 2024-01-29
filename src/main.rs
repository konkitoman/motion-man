use std::error::Error;

use motion_man::{
    color::Color,
    engine::Engine,
    rect::{RectBuilder, RectNodeManager},
};

use crate::{
    audio::{AudioBuilder, AudioNodeManager},
    backend::Backend,
    media::Media,
    video::{VideoBuilder, VideoNodeManager},
};

fn main() -> Result<(), Box<dyn Error>> {
    // tokio manual setup! :)
    // this is because we use single threaded!
    // i don't think this will work on a multithreaded application because i use `try_read and try_write`
    // because is outside of async
    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let _enter = rt.enter();

    // With this we create ower video engine 60 fps 1920x1080, audio 48KHz, 2 channels
    let mut engine = Engine::new(60., 1920.try_into()?, 1080.try_into()?, 48000, 2);

    // Here we register the nodes that we will need!

    engine.register_node::<RectNodeManager>();
    engine.register_node::<VideoNodeManager>();
    engine.register_node::<AudioNodeManager>();

    // This is the video that will create!
    engine.create_scene(|scene| {
        Box::pin(async move {
            scene
                .info(|info| {
                    println!("FPS: {}", info.fps());
                    println!("Width: {}", info.width);
                    println!("Height: {}", info.height);
                })
                .await;

            let fps = scene.fps(); // = 60

            // we create a rect bigger as the screen with the red color!
            let mut rect = scene.spawn(RectBuilder::new([1., 1.], Color::RED)).await;

            // `scene.present()` will render that many frames!
            //this is to see the red rectangle
            scene.present(fps / 2).await;

            // the set will call `scene.update()` that will update every node manager
            rect.color.set(Color::GREEN).await;

            scene.present(1).await;

            let mut rect2 = scene
                .spawn(RectBuilder::new([0.5, 0.5], Color::BLUE).with_position([-0.5, -0.5]))
                .await;

            scene.present(1).await;

            //                       from          to      time
            rect2.position.tween([-0.5, -0.5], [0.5, -0.5], 1.0).await;
            rect2.position.tween([0.5, -0.5], [0.5, 0.5], 1.0).await;
            rect2.position.tween([0.5, 0.5], [-0.5, 0.5], 1.0).await;
            rect2.position.tween([-0.5, 0.5], [-0.5, -0.5], 1.0).await;
            rect2.position.tween([-0.5, -0.5], [0., 0.], 1.0).await;

            // Play a video if is avalibile!
            if let Ok(mut media) = Media::new("video.mkv") {
                let mut video = scene
                    .spawn(VideoBuilder::new(media.video(0).unwrap()))
                    .await;
                let audio = scene
                    .spawn(AudioBuilder::new(media.audio(0).unwrap()))
                    .await;

                video.size.tween([0., 0.], [1., 1.], 1.0).await;

                while media.next() {
                    scene.present(1).await;
                }

                video.size.tween([1., 1.], [0.1, 0.1], 1.0).await;

                audio.drop().await;
                video.drop().await;
            }

            rect2.size.tween([0.5, 0.5], [0., 0.], 1.0).await;
            // this is a custom drop that will send a drop signal to the node manager then i will call `scene.update()`
            //  this will remove the node from the node manager, and will be allow to safely drop
            // if this is not called the engine will panic or abort!
            rect2.drop().await;

            rect.size.tween([1., 1.], [0., 0.], 1.0).await;
            rect.drop().await;
        })
    });

    // This is the backend
    let backend = Backend::new(engine, rt);

    // This will show a window, you can press Space to play/pause
    backend.preview();

    Ok(())
}

mod backend {
    use std::rc::Rc;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::time::Instant;

    use cpal::traits::DeviceTrait;
    use cpal::traits::HostTrait;
    use cpal::traits::StreamTrait;
    use glutin::config::Config;
    use glutin::config::ConfigTemplateBuilder;
    use glutin::config::GlConfig;
    use glutin::context::ContextAttributes;
    use glutin::context::ContextAttributesBuilder;
    use glutin::context::NotCurrentGlContext;
    use glutin::context::PossiblyCurrentContext;
    use glutin::display::Display;
    use glutin::display::GetGlDisplay;
    use glutin::display::GlDisplay;
    use glutin::surface::GlSurface;
    use glutin::surface::Surface;
    use glutin::surface::SurfaceAttributes;
    use glutin::surface::SurfaceAttributesBuilder;
    use glutin::surface::WindowSurface;
    use glutin_winit::DisplayBuilder;
    use motion_man::engine::Engine;
    use motion_man::gcx::GCX;
    use raw_window_handle::HasRawWindowHandle;
    use tokio::runtime::Runtime;
    use winit::event::Event;
    use winit::event::WindowEvent;
    use winit::event_loop::EventLoopBuilder;
    use winit::event_loop::EventLoopWindowTarget;
    use winit::keyboard::KeyCode;
    use winit::keyboard::PhysicalKey;
    use winit::window::Window;
    use winit::window::WindowBuilder;

    pub struct Backend {
        engine: Engine,
        rt: Runtime,
    }

    impl Backend {
        pub fn new(engine: Engine, rt: Runtime) -> Self {
            Self { engine, rt }
        }

        pub fn preview(mut self) {
            let event_loop = EventLoopBuilder::new().build().unwrap();
            let config_picker = |config: Box<dyn Iterator<Item = Config> + '_>| {
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
                        Config::Glx(_) => println!("  Backend: GLX"),
                        _ => {
                            println!("  Backend: Unknown")
                        }
                    }
                    index = i;
                }
                println!("Was selected: {index}");
                config
            };

            let audio_sender;

            ffmpeg_next::init().unwrap();
            let version = ffmpeg_next::util::version();
            println!("FFMPEG: {version}");

            {
                ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Trace);
                ffmpeg_next::log::set_flags(ffmpeg_next::log::Flags::SKIP_REPEATED);
            }

            println!("Hosts: {:?}", cpal::ALL_HOSTS);

            let host = cpal::host_from_id(cpal::ALL_HOSTS[0]).unwrap();
            let output = host.default_output_device().unwrap();

            let name = output.name().unwrap();
            println!("Output Device Name: {name}");

            let configs = output.supported_output_configs().unwrap();
            for config in configs {
                println!("Config: {config:?}");
            }

            let config = output.default_output_config().unwrap();
            println!("Default Output Config: {config:?}");

            let config = cpal::SupportedStreamConfig::new(
                2,
                cpal::SampleRate(48000),
                config.buffer_size().clone(),
                cpal::SampleFormat::F32,
            );

            println!("Using config: {config:?}");

            let (sender, receiver) = channel::<Vec<f32>>();
            audio_sender = sender;

            let mut buffer = Vec::<f32>::new();

            let stream = output
                .build_output_stream(
                    &config.config(),
                    move |out: &mut [f32], _callback_info| {
                        while let Ok(buf) = receiver.try_recv() {
                            buffer.extend(buf);
                        }

                        let mut tmp = buffer
                            .drain(..out.len().min(buffer.len()))
                            .collect::<Vec<f32>>();
                        tmp.resize(out.len(), 0.);

                        for (i, s) in tmp.into_iter().enumerate() {
                            out[i] = s;
                        }
                    },
                    |err| {
                        println!("Audio Error: {err:?}");
                    },
                    None,
                )
                .unwrap();

            stream.play().unwrap();

            pub struct Ctx {
                config: Config,
                display: Display,

                context_attributes: ContextAttributes,
                context: PossiblyCurrentContext,

                window: Window,
                surface_attributes: SurfaceAttributes<WindowSurface>,
                surface: Surface<WindowSurface>,

                gcx: GCX,
            }

            let mut ctx: Option<Ctx> = None;

            fn init_ctx(
                event_loop: &EventLoopWindowTarget<()>,
                config_picker: &dyn Fn(Box<dyn Iterator<Item = Config> + '_>) -> Config,
            ) -> Ctx {
                let (_, config) = DisplayBuilder::new()
                    .with_window_builder(None)
                    .build(&event_loop, ConfigTemplateBuilder::new(), config_picker)
                    .unwrap();
                let window = glutin_winit::finalize_window(
                    event_loop,
                    WindowBuilder::new().with_title("Motion Man Preview"),
                    &config,
                )
                .unwrap();
                let display = config.display();
                let surface_attributes;
                {
                    let size = window.inner_size();
                    surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                        window.raw_window_handle(),
                        size.width.try_into().unwrap(),
                        size.height.try_into().unwrap(),
                    );
                }
                let surface = unsafe {
                    display
                        .create_window_surface(&config, &surface_attributes)
                        .unwrap()
                };

                let context_attributes =
                    ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
                let context = unsafe {
                    display
                        .create_context(&config, &context_attributes)
                        .unwrap()
                };

                let context = context.make_current(&surface).unwrap();
                surface
                    .set_swap_interval(&context, glutin::surface::SwapInterval::DontWait)
                    .unwrap();

                let gl = unsafe {
                    glow::Context::from_loader_function_cstr(|addr| display.get_proc_address(addr))
                };

                let gcx = GCX::new(Rc::new(gl));

                Ctx {
                    config,
                    display,
                    context_attributes,
                    context,
                    window,
                    surface_attributes,
                    surface,
                    gcx,
                }
            }

            let mut running = false;
            let mut last = Instant::now();

            event_loop
                .run(|event, event_loop| 'lop: {
                    //
                    match event {
                        Event::NewEvents(cause) => match cause {
                            winit::event::StartCause::ResumeTimeReached {
                                start,
                                requested_resume,
                            } => {
                                let Some(ctx) = &mut ctx else { break 'lop };
                                if !running {
                                    event_loop
                                        .set_control_flow(winit::event_loop::ControlFlow::Wait);
                                    break 'lop;
                                }
                                ctx.window.request_redraw();
                                event_loop.set_control_flow(
                                    winit::event_loop::ControlFlow::WaitUntil(
                                        Instant::now()
                                            + Duration::from_secs_f64(
                                                self.engine.info.try_read().unwrap().delta,
                                            ),
                                    ),
                                );
                            }
                            winit::event::StartCause::WaitCancelled {
                                start,
                                requested_resume,
                            } => {
                                if !running {
                                    break 'lop;
                                }
                                if let Some(resume) = requested_resume {
                                    event_loop.set_control_flow(
                                        winit::event_loop::ControlFlow::WaitUntil(resume),
                                    );
                                }
                            }
                            _ => {}
                        },
                        Event::WindowEvent { window_id, event } => match event {
                            WindowEvent::RedrawRequested => {
                                last = Instant::now();
                                if self.engine.finished() {
                                    event_loop.exit();
                                }
                                let Some(ctx) = &mut ctx else { break 'lop };
                                self.rt.block_on(self.engine.run(&ctx.gcx));
                                self.engine.render(&ctx.gcx);
                                audio_sender
                                    .send(self.engine.audio_buffer().to_vec())
                                    .unwrap();
                                ctx.surface.swap_buffers(&ctx.context).unwrap();
                            }
                            WindowEvent::Resized(size) => {
                                let Some(ctx) = &mut ctx else { break 'lop };
                                ctx.surface.resize(
                                    &ctx.context,
                                    size.width.try_into().unwrap(),
                                    size.height.try_into().unwrap(),
                                );
                                ctx.gcx
                                    .viewport(0, 0, size.width as i32, size.height as i32);
                            }
                            WindowEvent::CloseRequested => {
                                std::process::abort();
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                let Some(ctx) = &mut ctx else { break 'lop };
                                match event.physical_key {
                                    PhysicalKey::Code(KeyCode::KeyL) => {
                                        if !event.state.is_pressed() {
                                            ctx.window.request_redraw();
                                        }
                                    }
                                    PhysicalKey::Code(KeyCode::Space) => {
                                        if !event.state.is_pressed() {
                                            running = !running;
                                            last = Instant::now();

                                            if running {
                                                event_loop.set_control_flow(
                                                    winit::event_loop::ControlFlow::WaitUntil(
                                                        last + Duration::from_secs_f64(
                                                            self.engine
                                                                .info
                                                                .try_read()
                                                                .unwrap()
                                                                .delta,
                                                        ),
                                                    ),
                                                )
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        },
                        Event::UserEvent(_) => todo!(),
                        Event::Suspended => {
                            _ = ctx.take();
                        }
                        Event::Resumed => {
                            let tmp_ctx = init_ctx(event_loop, &config_picker);
                            self.engine.init(&tmp_ctx.gcx);
                            ctx = Some(tmp_ctx);
                        }
                        Event::LoopExiting => {
                            println!("EventLoop Exiting!")
                        }
                        _ => {}
                    }
                })
                .unwrap()
        }
    }
}

mod video {
    use motion_man::{
        gcx::{
            buffer::{BufferType, BufferUsage},
            shader::{Shader, ShaderBuilder},
            texture::{Format, InternalFormat, Texture, TextureTarget, TextureType},
            vertex_array::{Field, Fields, VertexArray},
            DataType, GCX,
        },
        node::NodeBuilder,
        node::NodeManager,
        scene::SceneTask,
        signal::{create_signal, NSignal, RawSignal, Signal},
    };

    use crate::media::Stream;

    pub struct Video<'a> {
        pub position: Signal<'a, [f32; 2]>,
        pub size: Signal<'a, [f32; 2]>,

        scene: &'a SceneTask,

        drop: Signal<'a, ()>,
        dropped: bool,
    }

    pub struct RawVideo {
        position: RawSignal<[f32; 2]>,
        size: RawSignal<[f32; 2]>,

        drop: RawSignal<()>,
    }

    impl<'a> Drop for Video<'a> {
        fn drop(&mut self) {
            if self.dropped {
                return;
            }
            eprintln!("You need to call on a Video, drop() when is no more needed");
            std::process::abort();
        }
    }

    impl<'a> Video<'a> {
        pub async fn drop(mut self) {
            self.drop.set(()).await;
            self.dropped = true;
        }
    }

    pub struct VideoBuilder {
        stream: Box<dyn Stream>,
        size: [f32; 2],
        pos: [f32; 2],
    }

    impl VideoBuilder {
        pub fn new(stream: Box<dyn Stream>) -> Self {
            Self {
                stream,
                size: [1., 1.],
                pos: [0., 0.],
            }
        }
    }

    impl NodeBuilder for VideoBuilder {
        type Node<'a> = Video<'a>;
        type NodeManager = VideoNodeManager;

        fn create_node_ref<'a>(
            &self,
            RawVideo {
                position,
                size,
                drop,
            }: RawVideo,
            scene: &'a SceneTask,
        ) -> Self::Node<'a> {
            Video {
                scene,
                dropped: false,
                position: Signal::new(position, scene, self.pos),
                size: Signal::new(size, scene, self.size),
                drop: Signal::new(drop, scene, ()),
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct Vertex {
        position: [f32; 2],
        uv: [f32; 2],
    }

    impl Fields for Vertex {
        fn fields() -> Vec<motion_man::gcx::vertex_array::Field> {
            vec![
                Field::new::<[f32; 2]>("position"),
                Field::new::<[f32; 2]>("uv"),
            ]
        }
    }

    impl Vertex {
        pub fn new(x: f32, y: f32, uvx: f32, uvy: f32) -> Vertex {
            Vertex {
                position: [x, y],
                uv: [uvx, uvy],
            }
        }
    }

    struct RVideo {
        va: VertexArray,
        builder: VideoBuilder,
        texture: Option<Texture>,
        stream: Box<dyn Stream>,
        inner: RVideoInner,
    }

    impl RVideo {
        pub fn new(inner: RVideoInner, va: VertexArray, _gcx: &GCX, builder: VideoBuilder) -> Self {
            Self {
                va,
                stream: builder.stream.clone_ref(),
                builder,
                texture: None,
                inner,
            }
        }
    }

    pub struct RVideoInner {
        position: NSignal<[f32; 2]>,
        size: NSignal<[f32; 2]>,
        drop: NSignal<()>,
    }

    #[derive(Default)]
    pub struct VideoNodeManager {
        videos: Vec<RVideo>,
        shader: Option<Shader>,

        pending: Option<RVideoInner>,
    }

    impl NodeManager for VideoNodeManager {
        type NodeBuilder = VideoBuilder;
        type RawNode = RawVideo;

        fn init(&mut self, gcx: &motion_man::gcx::GCX) {
            self.shader.replace(
                ShaderBuilder::new()
                    .vertex(
                        r#"#version 320 es
                precision highp float;

                in vec2 pos;
                in vec2 uv;
                out vec2 UV;

                void main(){
                    gl_Position = vec4(pos, 0.0, 1.0);
                    UV = uv;
                }
                "#,
                    )
                    .fragment(
                        r#"#version 320 es
                precision highp float;

                uniform sampler2D IMAGE;

                out vec4 color;

                in vec2 UV;

                void main(){
                    color = vec4(texture(IMAGE, UV));
                }"#,
                    )
                    .build(gcx)
                    .unwrap(),
            );
        }

        fn init_node(&mut self, gcx: &motion_man::gcx::GCX, builder: Self::NodeBuilder) {
            let buffer = gcx.create_buffer(
                BufferType::ArrayBuffer,
                &create_mesh(&builder),
                BufferUsage::DRAW_STATIC,
            );
            let va = gcx.create_vertex_array::<Vertex>(buffer).build(gcx);

            self.videos
                .push(RVideo::new(self.pending.take().unwrap(), va, gcx, builder));
        }

        fn create_node(&mut self) -> RawVideo {
            let (sposition, position) = create_signal();
            let (ssize, size) = create_signal();
            let (sdrop, drop) = create_signal();

            self.pending = Some(RVideoInner {
                position,
                size,
                drop,
            });

            RawVideo {
                position: sposition,
                size: ssize,
                drop: sdrop,
            }
        }

        fn update(&mut self) {
            self.videos.retain_mut(|video| {
                let mut rebuild = false;

                if let Some(position) = video.inner.position.get() {
                    video.builder.pos = position;
                    rebuild = true;
                }

                if let Some(size) = video.inner.size.get() {
                    video.builder.size = size;
                    rebuild = true;
                }

                if video.inner.drop.get().is_some() {
                    return false;
                }

                if rebuild {
                    video
                        .va
                        .array_buffer
                        .update(0, &create_mesh(&video.builder));
                }

                true
            });
        }

        fn render(&mut self, gcx: &motion_man::gcx::GCX) {
            let shader = self.shader.as_ref().unwrap();
            gcx.use_shader(shader, |gcx| {
                for video in self.videos.iter_mut() {
                    gcx.use_vertex_array(&video.va, |gcx| {
                        if let Some(data) = video.stream.data(0) {
                            if let Some(texture) = &mut video.texture {
                                texture.update(0, data)
                            } else {
                                let width = video.stream.width().unwrap();
                                let height = video.stream.height().unwrap();
                                video.texture = Some(gcx.create_texture(
                                    TextureType::Tex2D,
                                    TextureTarget::Tex2D,
                                    0,
                                    InternalFormat::RGBA8,
                                    width as i32,
                                    height as i32,
                                    Format::RGBA,
                                    DataType::U8,
                                    data,
                                ));
                            }
                        }

                        let Some(texture) = &video.texture else {
                            return;
                        };
                        texture.activate(0);
                        shader.set_uniform("IMAGE", 0).unwrap();
                        gcx.draw_arrays(motion_man::gcx::PrimitiveType::TrianglesFan, 0, 4);
                    });
                }
            });
        }
    }

    fn create_mesh(builder: &VideoBuilder) -> [Vertex; 4] {
        [
            Vertex::new(
                -builder.size[0] + builder.pos[0],
                -builder.size[1] + builder.pos[1],
                0.0,
                1.0,
            ),
            Vertex::new(
                -builder.size[0] + builder.pos[0],
                builder.size[1] + builder.pos[1],
                0.0,
                0.0,
            ),
            Vertex::new(
                builder.size[0] + builder.pos[0],
                builder.size[1] + builder.pos[1],
                1.0,
                0.0,
            ),
            Vertex::new(
                builder.size[0] + builder.pos[0],
                -builder.size[1] + builder.pos[1],
                1.0,
                1.0,
            ),
        ]
    }
}

mod media {
    use std::{any::Any, path::Path, sync::Arc};

    use tokio::sync::RwLock;

    use ffmpeg::{
        codec::Parameters,
        format::context::Input as FInput,
        format::{input as finput, Pixel},
        frame::Audio as AFrame,
        frame::Video as VFrame,
        util::error::Error as AVError,
        ChannelLayout, Packet,
    };
    use ffmpeg_next as ffmpeg;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum StreamType {
        Video,
        Audio,
    }

    pub trait Stream: Send + Sync {
        fn ty(&self) -> StreamType;
        fn stream_index(&self) -> usize;
        fn clone_ref(&self) -> Box<dyn Stream>;

        fn send_packet(&self, decoder: &mut Box<dyn Any>, packet: Packet);

        fn next(&self) -> bool;
        fn prev(&self) -> bool;
        fn clear(&self);

        fn data(&self, index: usize) -> Option<&[u8]>;

        fn audio_buffer(&self, from: usize) -> Option<Vec<f32>> {
            None
        }

        fn samples(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> Option<usize> {
            None
        }

        fn current(&self) -> usize;

        fn width(&self) -> Option<u32> {
            None
        }
        fn height(&self) -> Option<u32> {
            None
        }

        fn gc(&self);
    }

    pub struct VideoStream {
        stream_index: usize,

        frames: Vec<VFrame>,
        index: usize,
        current: usize,
    }

    impl VideoStream {
        fn new(index: usize) -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self {
                frames: Vec::default(),
                index: usize::MAX,
                stream_index: index,
                current: 0,
            }))
        }
    }

    impl Stream for Arc<RwLock<VideoStream>> {
        fn ty(&self) -> StreamType {
            StreamType::Video
        }

        fn stream_index(&self) -> usize {
            self.blocking_read().stream_index
        }

        fn clone_ref(&self) -> Box<dyn Stream> {
            Box::new(self.clone())
        }

        fn send_packet(&self, decoder: &mut Box<dyn Any>, packet: Packet) {
            let decoder = decoder.downcast_mut::<VideoDecoder>().unwrap();
            decoder.decoder.send_packet(&packet).unwrap();
            let mut frame = VFrame::empty();
            if decoder.decoder.receive_frame(&mut frame).is_err() {
                return;
            }

            let mut dst = VFrame::new(Pixel::RGBA, frame.width(), frame.height());
            let mut sws = frame.converter(Pixel::RGBA).unwrap();
            sws.run(&frame, &mut dst).unwrap();

            let s = &mut *self.try_write().unwrap();

            s.frames.push(dst);
        }

        fn next(&self) -> bool {
            self.gc();
            let s = &mut *self.try_write().unwrap();

            if !s.frames.is_empty() && s.index == usize::MAX {
                s.index = 0;
                return true;
            }

            if s.index < s.frames.len() {
                s.index += 1;
                true
            } else {
                false
            }
        }

        fn prev(&self) -> bool {
            let s = &mut *self.try_write().unwrap();

            if s.index > 0 {
                s.index -= 1;
                true
            } else {
                false
            }
        }

        fn gc(&self) {
            let s = &mut *self.try_write().unwrap();

            if s.index > 100 && s.index != usize::MAX {
                s.frames.drain(..50);
                s.index -= 50;
            }
        }

        fn clear(&self) {
            let s = &mut *self.try_write().unwrap();
            s.index = usize::MAX;
            s.frames.clear();
        }

        fn data(&self, index: usize) -> Option<&[u8]> {
            let s = &*self.try_read().unwrap();

            if s.index == usize::MAX {
                return None;
            }

            let f = &s.frames[s.index];
            let data = unsafe {
                core::slice::from_raw_parts(
                    (*f.as_ptr()).data[index],
                    f.stride(index) * f.plane_height(index) as usize,
                )
            };
            Some(data)
        }

        fn width(&self) -> Option<u32> {
            let s = &*self.try_read().unwrap();
            if s.index == usize::MAX {
                return None;
            }
            Some(s.frames[s.index].width())
        }

        fn height(&self) -> Option<u32> {
            let s = &*self.try_read().unwrap();
            if s.index == usize::MAX {
                return None;
            }
            Some(s.frames[s.index].height())
        }

        fn current(&self) -> usize {
            self.try_read().unwrap().current
        }
    }

    pub struct AudioStream {
        frames: Vec<AFrame>,
        index: usize,
        current: usize,
        forword: bool,

        stream_index: usize,
    }

    impl AudioStream {
        pub fn new(index: usize) -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self {
                frames: Vec::default(),
                index: usize::MAX,
                stream_index: index,
                current: 0,
                forword: false,
            }))
        }
    }

    impl Stream for Arc<RwLock<AudioStream>> {
        fn ty(&self) -> StreamType {
            StreamType::Audio
        }

        fn stream_index(&self) -> usize {
            self.try_read().unwrap().stream_index
        }

        fn audio_buffer(&self, from: usize) -> Option<Vec<f32>> {
            let mut buffer = Vec::<f32>::with_capacity(self.samples()?);

            let s = &*self.try_read().unwrap();

            let diff = s.current - from;

            for i in 0..diff {
                let index = if s.forword {
                    s.index - (diff - i)
                } else {
                    s.index + (diff - i)
                };
                if let Some(frame) = s.frames.get(index) {
                    let mut plane1 = frame.plane::<f32>(0)[..].iter();
                    let mut plane2 = frame.plane::<f32>(1)[..].iter();
                    let mut state = true;

                    buffer.extend(core::iter::from_fn(move || {
                        if state {
                            state = false;
                            plane1.next()
                        } else {
                            state = true;
                            plane2.next()
                        }
                    }));
                } else {
                    eprintln!("No frame for: {index}");
                }
            }

            Some(buffer)
        }

        fn send_packet(&self, decoder: &mut Box<dyn Any>, packet: Packet) {
            let decoder = decoder.downcast_mut::<AudioDecoder>().unwrap();
            decoder.decoder.send_packet(&packet).unwrap();
            let mut frame = AFrame::empty();
            if decoder.decoder.receive_frame(&mut frame).is_err() {
                return;
            }

            let mut dst = AFrame::new(
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
                frame.samples(),
                ChannelLayout::STEREO,
            );
            let mut sws = frame
                .resampler(
                    ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
                    ChannelLayout::STEREO,
                    48000,
                )
                .unwrap();
            let delay = sws.run(&frame, &mut dst).unwrap();

            self.try_write().unwrap().frames.push(dst);
        }

        fn clone_ref(&self) -> Box<dyn Stream> {
            Box::new(self.clone())
        }

        fn next(&self) -> bool {
            self.gc();
            let s = &mut *self.try_write().unwrap();
            s.forword = true;

            if !s.frames.is_empty() && s.index == usize::MAX {
                s.index = 0;
                return true;
            }

            if s.index < s.frames.len() {
                s.current += 1;
                s.index += 1;
                true
            } else {
                false
            }
        }

        fn prev(&self) -> bool {
            let s = &mut *self.try_write().unwrap();
            s.forword = false;

            if s.index > 0 {
                s.current += 1;
                s.index -= 1;
                true
            } else {
                false
            }
        }

        fn gc(&self) {
            let s = &mut *self.try_write().unwrap();

            if s.index > 100 && s.index != usize::MAX {
                s.frames.drain(..50);
                s.index -= 50;
            }
        }

        fn clear(&self) {
            let s = &mut *self.try_write().unwrap();
            s.index = usize::MAX;
            s.frames.clear();
        }

        fn data(&self, index: usize) -> Option<&[u8]> {
            let s = &*self.blocking_read();

            if s.index == usize::MAX {
                return None;
            }

            let f = &s.frames[s.index];
            let data = unsafe {
                core::slice::from_raw_parts(
                    (*f.as_ptr()).data[index],
                    (*f.as_ptr()).linesize[index] as usize,
                )
            };
            Some(data)
        }

        fn samples(&self) -> Option<usize> {
            let s = &*self.blocking_read();

            if s.index == usize::MAX {
                return None;
            }

            let f = &s.frames[s.index];
            Some(f.samples())
        }

        fn channels(&self) -> Option<usize> {
            let s = &*self.blocking_read();

            if s.index == usize::MAX {
                return None;
            }

            let f = &s.frames[s.index];
            Some(f.channels() as usize)
        }

        fn current(&self) -> usize {
            self.try_read().unwrap().current
        }
    }

    pub struct VideoDecoder {
        decoder: ffmpeg::codec::decoder::Video,
    }

    impl VideoDecoder {
        pub fn new<D: ffmpeg::codec::traits::Decoder>(params: Parameters, codec: D) -> Self {
            let mut ctx = ffmpeg::codec::Context::new();
            ctx.set_parameters(params).unwrap();
            let decoder = ctx.decoder().open_as(codec).unwrap().video().unwrap();
            Self { decoder }
        }
    }

    struct AudioDecoder {
        decoder: ffmpeg::codec::decoder::Audio,
    }

    impl AudioDecoder {
        pub fn new<D: ffmpeg::codec::traits::Decoder>(params: Parameters, codec: D) -> Self {
            let mut ctx = ffmpeg::codec::Context::new();
            ctx.set_parameters(params).unwrap();
            let decoder = ctx.decoder().open_as(codec).unwrap().audio().unwrap();
            Self { decoder }
        }
    }

    pub struct Media {
        format: FInput,

        streams: Vec<Box<dyn Stream>>,
        decoders: Vec<Box<dyn Any>>,
    }

    unsafe impl Send for Media {}
    unsafe impl Sync for Media {}

    impl Media {
        pub fn new<P: AsRef<Path>>(url: P) -> Result<Self, AVError> {
            let format = finput(&url)?;

            let mut streams = Vec::<Box<dyn Stream>>::default();
            let mut decoders = Vec::<Box<dyn Any>>::default();

            for (i, stream) in format.streams().enumerate() {
                match stream.parameters().medium() {
                    ffmpeg::media::Type::Unknown => todo!(),
                    ffmpeg::media::Type::Video => {
                        let s = VideoStream::new(i);
                        let decoder =
                            VideoDecoder::new(stream.parameters(), stream.parameters().id());
                        decoders.push(Box::new(decoder));
                        streams.push(Box::new(s));
                    }
                    ffmpeg::media::Type::Audio => {
                        let s = AudioStream::new(i);
                        let decoder =
                            AudioDecoder::new(stream.parameters(), stream.parameters().id());
                        decoders.push(Box::new(decoder));
                        streams.push(Box::new(s));
                    }
                    ffmpeg::media::Type::Data => todo!(),
                    ffmpeg::media::Type::Subtitle => todo!(),
                    ffmpeg::media::Type::Attachment => todo!(),
                }
            }

            Ok(Self {
                format,
                streams,
                decoders,
            })
        }

        pub fn video(&self, index: usize) -> Option<Box<dyn Stream>> {
            let mut i = 0;
            for stream in self.streams.iter() {
                if stream.ty() != StreamType::Video {
                    continue;
                }

                if i == index {
                    return Some(stream.clone_ref());
                }

                i += 1;
            }

            None
        }

        pub fn audio(&self, index: usize) -> Option<Box<dyn Stream>> {
            let mut i = 0;
            for stream in self.streams.iter() {
                if stream.ty() != StreamType::Audio {
                    continue;
                }

                if i == index {
                    return Some(stream.clone_ref());
                }

                i += 1;
            }

            None
        }

        pub fn next(&mut self) -> bool {
            let mut readys = vec![false; self.streams.len()];
            loop {
                let Some((stream, packet)) = self.format.packets().next() else {
                    return false;
                };

                let i = stream.index();

                let decoder = &mut self.decoders[i];
                self.streams[i].send_packet(decoder, packet);
                let ready = self.streams[i].next();
                if !readys[i] {
                    readys[i] = ready;
                }

                if readys
                    .iter()
                    .fold(true, |val, ready| if !*ready { false } else { val })
                {
                    break;
                }
            }
            true
        }
    }
}

mod audio {
    use motion_man::{
        node::NodeBuilder,
        node::NodeManager,
        signal::{create_signal, NSignal, RawSignal, Signal},
    };

    use crate::media::Stream;

    pub struct Audio<'a> {
        drop: Signal<'a, ()>,
        dropped: bool,
    }

    pub struct RawAudio {
        drop: RawSignal<()>,
    }

    impl<'a> Audio<'a> {
        pub async fn drop(mut self) {
            self.drop.set(()).await;
            self.dropped = true;
        }
    }

    impl<'a> Drop for Audio<'a> {
        fn drop(&mut self) {
            if !self.dropped {
                eprintln!("You need to call drop on Audio when is no more needed!");
                std::process::abort();
            }
        }
    }

    pub struct AudioBuilder {
        stream: Box<dyn Stream>,
    }

    impl AudioBuilder {
        pub fn new(stream: Box<dyn Stream>) -> Self {
            Self { stream }
        }
    }

    impl NodeBuilder for AudioBuilder {
        type Node<'a> = Audio<'a>;
        type NodeManager = AudioNodeManager;

        fn create_node_ref<'a>(
            &self,
            RawAudio { drop }: RawAudio,
            scene: &'a motion_man::scene::SceneTask,
        ) -> Self::Node<'a> {
            Audio {
                drop: Signal::new(drop, scene, ()),
                dropped: false,
            }
        }
    }

    #[derive(Default)]
    pub struct AudioNodeManager {
        audios: Vec<(NSignal<()>, Box<dyn Stream>, Vec<f32>, usize)>,
        pending: Option<NSignal<()>>,
    }

    impl NodeManager for AudioNodeManager {
        type NodeBuilder = AudioBuilder;
        type RawNode = RawAudio;

        fn init_node(&mut self, _gcx: &motion_man::gcx::GCX, builder: Self::NodeBuilder) {
            let drop = self.pending.take().unwrap();
            self.audios.push((drop, builder.stream, Vec::new(), 0));
        }

        fn create_node(&mut self) -> RawAudio {
            let (drop, ndrop) = create_signal::<()>();

            self.pending = Some(ndrop);

            RawAudio { drop }
        }

        fn update(&mut self) {
            self.audios.retain_mut(|audio| {
                if audio.0.get().is_some() {
                    return false;
                }
                true
            })
        }

        fn audio_process(&mut self, buffer: &mut [f32]) {
            for audio in self.audios.iter_mut() {
                if let Some(samples) = audio.1.audio_buffer(audio.3) {
                    audio.2.extend(samples);
                    audio.3 = audio.1.current();
                }

                let tmp = audio
                    .2
                    .drain(..buffer.len().min(audio.2.len()))
                    .collect::<Vec<f32>>();

                for i in 0..tmp.len() {
                    buffer[i] += tmp[i];
                }
            }
        }
    }
}
