use std::{
    error::Error,
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
    gcx::{BufferBit, GCX, GL},
    rect::{RectBuilder, RectNode},
};

use crate::video::{VideoBuilder, VideoNode};

pub enum SceneMessage {
    NextFrame,
    Resumed,
}

fn make_context(
    builder: WindowBuilder,
) -> Result<
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

    let window = glutin_winit::finalize_window(&event_loop, builder, &config).unwrap();

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
        create_cell,
        element::ElementBuilder,
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
        scene::SceneTask,
        signal::{create_signal, NSignal, Signal, SignalInner},
        RCell, SCell, SSAny,
    };

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
        type Element<'a> = Video<'a>;

        fn node_id(&self) -> std::any::TypeId {
            TypeId::of::<VideoNode>()
        }

        fn create_element_ref<'a>(
            &self,
            inner: Box<SSAny>,
            scene: &'a SceneTask,
        ) -> Self::Element<'a> {
            let (position, size, step, drop, finished): (
                SignalInner<[f32; 2]>,
                SignalInner<[f32; 2]>,
                SignalInner<()>,
                SignalInner<()>,
                RCell<bool>,
            ) = *inner.downcast().unwrap();

            Video {
                scene,
                droped: false,
                position: Signal::new(position, scene, self.pos),
                size: Signal::new(size, scene, self.size),
                step: Signal::new(step, scene, ()),
                drop: Signal::new(drop, scene, ()),
                finished,
            }
        }
    }

    pub struct Video<'a> {
        pub position: Signal<'a, [f32; 2]>,
        pub size: Signal<'a, [f32; 2]>,
        pub step: Signal<'a, ()>,
        pub finished: RCell<bool>,

        scene: &'a SceneTask,

        drop: Signal<'a, ()>,
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
        pub async fn drop(mut self) {
            self.drop.set(()).await;
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
        finished: bool,
        inner: RVideoInner,
    }

    impl RVideo {
        pub fn new(inner: RVideoInner, va: VertexArray, gcx: &GCX, builder: VideoBuilder) -> Self {
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
                AVFrame::with_image(frame.width(), frame.height(), AVPixelFormat::RGBA).unwrap();
            let sws = SwsContext::from_frame(&frame, &dst_frame);
            sws.sws_scale(&frame, &mut dst_frame).unwrap();

            let mut i = 0;
            let data = dst_frame.data();

            let texture = gcx.create_texture(
                TextureType::Tex2D,
                TextureTarget::Tex2D,
                0,
                InternalFormat::RGBA8,
                frame.width(),
                frame.height(),
                Format::RGBA,
                DataType::U8,
                &dst_frame.data()[0],
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
                finished: false,
                inner,
            }
        }
    }

    impl core::fmt::Debug for RVideo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    pub struct RVideoInner {
        finished: SCell<bool>,
        position: NSignal<[f32; 2]>,
        size: NSignal<[f32; 2]>,
        step: NSignal<()>,
        drop: NSignal<()>,
    }

    pub struct VideoNode {
        videos: Vec<RVideo>,
        shader: Option<Shader>,

        pending: Option<RVideoInner>,
    }

    impl Default for VideoNode {
        fn default() -> Self {
            Self {
                videos: Vec::new(),
                shader: None,
                pending: None,
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
                .push(RVideo::new(self.pending.take().unwrap(), va, gcx, builder));
        }

        fn create_element(&mut self) -> Box<SSAny> {
            let (sposition, position) = create_signal();
            let (ssize, size) = create_signal();
            let (sstep, step) = create_signal();
            let (sdrop, drop) = create_signal();
            let (finished, rfinished) = create_cell(false);

            self.pending = Some(RVideoInner {
                finished,
                position,
                size,
                step,
                drop,
            });
            Box::new((sposition, ssize, sstep, sdrop, rfinished))
        }

        fn update(&mut self) {
            self.videos.retain_mut(|video| {
                if let Some(_) = video.inner.step.get() {
                    'video: {
                        if video.finished {
                            break 'video;
                        }

                        loop {
                            if video.format_context.read_frame(&mut video.packet).is_ok() {
                                if video.packet.stream_index() == video.stream_index {
                                    video.codec_context.send_packet(&video.packet);
                                    if video.codec_context.receive_frame(&mut video.frame).is_ok() {
                                        break;
                                    }
                                }
                            } else {
                                video.inner.finished.set(true);
                                video.finished = true;
                                break 'video;
                            }
                        }

                        video
                            .sws
                            .sws_scale(&video.frame, &mut video.dst_frame)
                            .unwrap();

                        video.texture.update(0, video.dst_frame.data()[0]);
                    }
                }

                let mut rebuild = false;

                if let Some(position) = video.inner.position.get() {
                    video.builder.pos = position;
                    rebuild = true;
                }

                if let Some(size) = video.inner.size.get() {
                    video.builder.size = size;
                    rebuild = true;
                }

                if let Some(_) = video.inner.drop.get() {
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
    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let _enter = rt.enter();

    let mut engine = Engine::new(60., 1920.try_into()?, 1080.try_into()?);

    engine.register_node::<RectNode>();
    engine.register_node::<VideoNode>();

    engine.create_scene(|scene| {
        Box::pin(async move {
            scene
                .info(|info| {
                    println!("FPS: {}", info.fps());
                    println!("Width: {}", info.width);
                    println!("Height: {}", info.height);
                })
                .await;

            let fps = scene.fps();

            let mut rect = scene
                .spawn_element(RectBuilder::new([1., 1.], Color::RED))
                .await;

            scene.wait(fps / 2).await;

            rect.color.set(Color::GREEN).await;

            scene.update().await;
            scene.wait(1).await;

            let mut rect2 = scene
                .spawn_element(
                    RectBuilder::new([0.5, 0.5], Color::BLUE).with_position([-0.5, -0.5]),
                )
                .await;

            scene.wait(1).await;

            rect2.position.tween([-0.5, -0.5], [0.5, -0.5], 1.0).await;
            rect2.position.tween([0.5, -0.5], [0.5, 0.5], 1.0).await;
            rect2.position.tween([0.5, 0.5], [-0.5, 0.5], 1.0).await;
            rect2.position.tween([-0.5, 0.5], [-0.5, -0.5], 1.0).await;
            rect2.position.tween([-0.5, -0.5], [0., 0.], 1.0).await;

            let mut video = scene.spawn_element(VideoBuilder::new("video.mkv")).await;

            video.size.tween([0., 0.], [1., 1.], 1.0).await;

            while !video.finished.get() {
                video.step.set(()).await;
                scene.wait(1).await;
            }

            video.size.tween([1., 1.], [0.1, 0.1], 1.0).await;

            video.drop().await;

            rect2.size.tween([0.5, 0.5], [0., 0.], 1.0).await;
            rect2.drop().await;

            rect.size.tween([1., 1.], [0., 0.], 1.0).await;
            rect.drop().await;
        })
    });

    let width = engine.info.try_read().unwrap().width;
    let height = engine.info.try_read().unwrap().height;

    let (event_loop, window, config, context, surface, gl) =
        make_context(WindowBuilder::new().with_title("Motion Man"))?;
    let gcx = GCX::new(Rc::new(gl));
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
