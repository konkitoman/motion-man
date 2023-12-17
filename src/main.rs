use std::{
    borrow::Cow,
    error::Error,
    ffi::{CStr, CString},
    future::Future,
    path::Path,
    pin::Pin,
    rc::Rc,
    time::{Duration, Instant},
};

use glutin::{
    config::{Config, ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::HasRawWindowHandle;

use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};
use GL::HasContext;

use motion_man::{
    color::Color,
    engine::Engine,
    ffmpeg::*,
    gcx::{
        buffer::BufferUsage,
        texture::{Format, InternalFormat, TextureTarget, TextureType},
        BufferBit, GCX, GL,
    },
    rect::{RectBuilder, RectNode},
};

use crate::video::{VideoBuilder, VideoNode};

pub enum SceneMessage {
    NextFrame,
    Resumed,
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

mod video {

    use std::any::TypeId;

    use motion_man::{
        element::{ElementBuilder, ElementMessage},
        ffmpeg::{
            AVCodecContext, AVCodecType, AVFormatContext, AVFrame, AVPacket, AVPixelFormat,
            SwsContext,
        },
        gcx::{
            buffer::{BufferType, BufferUsage},
            shader::{Shader, ShaderBuilder},
            texture::{Format, InternalFormat, Texture, TextureTarget, TextureType},
            vertex_array::{Field, Fields, VertexArray},
            DataType, GCX,
        },
        node::Node,
        ochannel,
        scene::SceneTask,
        OSend,
    };
    use tokio::sync::mpsc::{channel, Receiver, Sender};

    #[derive(Debug)]
    pub struct VideoBuilder {
        url: String,
        size: [f32; 2],
        pos: [f32; 2],
    }

    impl VideoBuilder {
        pub fn new(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                size: [1., 1.],
                pos: [0., 0.],
            }
        }
    }

    impl ElementBuilder for VideoBuilder {
        type ElementRef<'a> = Video<'a>;

        fn node_id(&self) -> std::any::TypeId {
            TypeId::of::<VideoNode>()
        }

        fn create_element_ref<'a>(
            &self,
            sender: Sender<ElementMessage>,
            scene: &'a SceneTask,
        ) -> Self::ElementRef<'a> {
            Video {
                sender,
                scene,
                droped: false,
            }
        }
    }

    pub struct Video<'a> {
        sender: Sender<ElementMessage>,
        scene: &'a SceneTask,
        droped: bool,
    }

    impl<'a> Drop for Video<'a> {
        fn drop(&mut self) {
            if self.droped {
                return;
            }
            eprintln!("You need to call on a Video, drop() when is no more needed");
            std::process::abort();
        }
    }

    impl<'a> Video<'a> {
        pub async fn step(&self) {
            self.scene.submit().await;
            self.sender
                .send(ElementMessage::Set(0, Box::new(())))
                .await
                .unwrap();
            self.scene.submit().await;
        }

        pub async fn set_position(&self, pos: [f32; 2]) {
            self.scene.submit().await;
            self.sender
                .send(ElementMessage::Set(2, Box::new(pos)))
                .await
                .unwrap();
            self.scene.submit().await;
        }

        pub async fn set_size(&self, scale: [f32; 2]) {
            self.scene.submit().await;
            self.sender
                .send(ElementMessage::Set(3, Box::new(scale)))
                .await
                .unwrap();
            self.scene.submit().await;
        }

        pub async fn is_finished(&self) -> bool {
            let (sender, receiver) = ochannel::<bool>();
            self.scene.submit().await;
            self.sender
                .send(ElementMessage::Set(1, Box::new(sender)))
                .await
                .unwrap();
            self.scene.submit().await;
            receiver.await.unwrap()
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
        id: u64,
        va: VertexArray,
        builder: VideoBuilder,
        texture: Texture,
        format_context: AVFormatContext,
        codec_context: AVCodecContext,
        stream_index: i32,
        frame: AVFrame,
        dst_frame: AVFrame,
        packet: AVPacket,
        sws: SwsContext,
        buffer: Vec<u32>,
        finished: bool,
    }

    impl RVideo {
        pub fn new(id: u64, va: VertexArray, gcx: &GCX, builder: VideoBuilder) -> Self {
            let mut format_context = AVFormatContext::new(builder.url.clone()).unwrap();
            let streams = format_context.streams();

            let mut stream = None;
            let mut index = 0;
            for (i, tmp_stream) in streams.enumerate() {
                if let AVCodecType::Video = tmp_stream.codec_type() {
                    stream = Some(tmp_stream);
                    index = i;
                    break;
                }
            }

            let stream = stream.unwrap();

            let codec = stream.decoder().unwrap();

            let mut codec_context =
                AVCodecContext::with_params(&codec, &stream.codec_params()).unwrap();

            let mut packet = AVPacket::default();
            let mut frame = AVFrame::default();

            loop {
                format_context.read_frame(&mut packet).unwrap();
                if packet.stream_index() == index as i32 {
                    codec_context.send_packet(&packet);
                    if codec_context.receive_frame(&mut frame).is_ok() {
                        break;
                    }
                }
            }

            let mut dst_frame =
                AVFrame::with_image(frame.width(), frame.height(), AVPixelFormat::RGB24).unwrap();
            let sws = SwsContext::from_frame(&frame, &dst_frame);
            sws.sws_scale(&frame, &mut dst_frame).unwrap();

            let mut buffer = vec![0u32; frame.width() as usize * frame.height() as usize];

            let mut i = 0;
            let data = dst_frame.data();
            while i < data[0].len() {
                let r = data[0][i] as u32;
                let g = data[0][i + 1] as u32;
                let b = data[0][i + 2] as u32;
                let a = 255;
                buffer[i / 3] = r + (g << 8) + (b << 16) + (a << 24);
                i += 3;
            }

            let texture = gcx.create_texture(
                TextureType::Tex2D,
                TextureTarget::Tex2D,
                0,
                InternalFormat::RGBA8,
                frame.width(),
                frame.height(),
                Format::RGBA,
                DataType::U8,
                &buffer,
            );

            Self {
                va,
                builder,
                texture,
                format_context,
                codec_context,
                stream_index: index as i32,
                frame,
                dst_frame,
                packet,
                sws,
                buffer,
                id,
                finished: false,
            }
        }
    }

    impl core::fmt::Debug for RVideo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct VideoNode {
        videos: Vec<RVideo>,
        shader: Option<Shader>,

        counter: u64,
        receivers: Vec<(Receiver<ElementMessage>, u64)>,
    }

    impl Default for VideoNode {
        fn default() -> Self {
            Self {
                videos: Vec::new(),
                shader: None,
                receivers: Vec::new(),
                counter: 0,
            }
        }
    }

    impl Node for VideoNode {
        type ElementBuilder = VideoBuilder;

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

        fn init_element(&mut self, gcx: &motion_man::gcx::GCX, builder: Self::ElementBuilder) {
            let buffer = gcx.create_buffer(
                BufferType::ArrayBuffer,
                &create_mesh(&builder),
                BufferUsage::DRAW_STATIC,
            );
            let va = gcx.create_vertex_array::<Vertex>(buffer).build(gcx);

            self.videos
                .push(RVideo::new(self.counter, va, gcx, builder));
            self.counter += 1;
        }

        fn create_ref(&mut self) -> tokio::sync::mpsc::Sender<motion_man::element::ElementMessage> {
            let (sender, receiver) = channel(1);
            self.receivers.push((receiver, self.counter));
            sender
        }

        fn update(&mut self) {
            self.receivers.retain_mut(|(receiver, id)| {
                if let Ok(msg) = receiver.try_recv() {
                    match msg {
                        ElementMessage::Set(0, _) => {
                            'video: for video in self.videos.iter_mut() {
                                if video.id == *id {
                                    if video.finished {
                                        break;
                                    }

                                    loop {
                                        if video
                                            .format_context
                                            .read_frame(&mut video.packet)
                                            .is_ok()
                                        {
                                            if video.packet.stream_index() == video.stream_index {
                                                video.codec_context.send_packet(&video.packet);
                                                if video
                                                    .codec_context
                                                    .receive_frame(&mut video.frame)
                                                    .is_ok()
                                                {
                                                    break;
                                                }
                                            }
                                        } else {
                                            video.finished = true;
                                            break 'video;
                                        }
                                    }

                                    video
                                        .sws
                                        .sws_scale(&video.frame, &mut video.dst_frame)
                                        .unwrap();

                                    let mut i = 0;
                                    let data = video.dst_frame.data();
                                    while i < data[0].len() {
                                        let r = data[0][i] as u32;
                                        let g = data[0][i + 1] as u32;
                                        let b = data[0][i + 2] as u32;
                                        let a = 255;
                                        video.buffer[i / 3] = r + (g << 8) + (b << 16) + (a << 24);
                                        i += 3;
                                    }
                                    video.texture.update(0, &video.buffer);
                                    break;
                                }
                            }
                        }
                        ElementMessage::Set(1, send) => {
                            let send = send.downcast::<OSend<bool>>().unwrap();
                            for video in self.videos.iter() {
                                if video.id == *id {
                                    send.send(video.finished).unwrap();
                                    break;
                                }
                            }
                        }
                        ElementMessage::Set(2, pos) => {
                            let pos = pos.downcast::<[f32; 2]>().unwrap();
                            for video in self.videos.iter_mut() {
                                if video.id == *id {
                                    video.builder.pos = *pos;
                                    video
                                        .va
                                        .array_buffer
                                        .update(0, &create_mesh(&video.builder));
                                    break;
                                }
                            }
                        }
                        ElementMessage::Set(3, size) => {
                            let size = size.downcast::<[f32; 2]>().unwrap();
                            for video in self.videos.iter_mut() {
                                if video.id == *id {
                                    video.builder.size = *size;
                                    video
                                        .va
                                        .array_buffer
                                        .update(0, &create_mesh(&video.builder));
                                    break;
                                }
                            }
                        }
                        ElementMessage::Set(21, _) => {
                            self.videos.retain(|v| v.id != *id);
                            return false;
                        }
                        _ => {}
                    }
                }
                true
            });
        }

        fn render(&self, gcx: &motion_man::gcx::GCX) {
            let shader = self.shader.as_ref().unwrap();
            gcx.use_shader(shader, |gcx| {
                for video in self.videos.iter() {
                    gcx.use_vertex_array(&video.va, |gcx| {
                        video.texture.activate(0);
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

fn main() -> Result<(), Box<dyn Error>> {
    let (event_loop, window, config, context, surface, gl) = make_context()?;

    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let _enter = rt.enter();

    let mut engine = Engine::new(60., 1920.try_into()?, 1080.try_into()?);

    engine.register_node::<RectNode>();
    engine.register_node::<VideoNode>();

    engine.create_scene(|mut scene| {
        Box::pin(async move {
            scene
                .info(|info| {
                    println!("FPS: {}", info.fps());
                    println!("Width: {}", info.width);
                    println!("Height: {}", info.height);
                })
                .await;

            let rect = scene
                .spawn_element(RectBuilder::new([1., 1.], Color::RED))
                .await;

            scene.wait(scene.info(|i| i.fps()).await).await;

            rect.set_color(Color::GREEN);

            scene.submit().await;
            scene.wait(1).await;

            let rect2 = scene
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

            let video = scene.spawn_element(VideoBuilder::new("video.mkv")).await;

            let mut s = 0.0;

            while !video.is_finished().await {
                video.set_size([s, s]).await;
                video.step().await;
                scene.submit().await;
                s += scene.info(|i| i.delta).await as f32 / 2.;
                s = s.clamp(0.0, 1.0);
                scene.wait(1).await;
            }

            for _ in 0..120 {
                video.set_size([s, s]).await;
                s -= scene.info(|i| i.delta).await as f32 / 2.;
                s = s.clamp(0.0, 1.0);
                scene.wait(1).await;
            }

            video.drop().await;

            scene.wait(60).await;
            rect2.drop().await;
            scene.wait(60).await;
            rect.drop().await;
            scene.wait(10).await;
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
