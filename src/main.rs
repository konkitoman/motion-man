use std::{
    error::Error,
    rc::Rc,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate,
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

use crate::{
    audio::{AudioBuilder, AudioNode},
    media::Media,
    video::{VideoBuilder, VideoNode},
};

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

    use crate::media::Stream;

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
            let (position, size, drop): (
                SignalInner<[f32; 2]>,
                SignalInner<[f32; 2]>,
                SignalInner<()>,
            ) = *inner.downcast().unwrap();

            Video {
                scene,
                droped: false,
                position: Signal::new(position, scene, self.pos),
                size: Signal::new(size, scene, self.size),
                drop: Signal::new(drop, scene, ()),
            }
        }
    }

    pub struct Video<'a> {
        pub position: Signal<'a, [f32; 2]>,
        pub size: Signal<'a, [f32; 2]>,

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
        texture: Option<Texture>,
        stream: Box<dyn Stream>,
        inner: RVideoInner,
    }

    impl RVideo {
        pub fn new(inner: RVideoInner, va: VertexArray, gcx: &GCX, builder: VideoBuilder) -> Self {
            return Self {
                va,
                stream: builder.stream.clone_ref(),
                builder,
                texture: None,
                inner,
            };
        }
    }

    pub struct RVideoInner {
        position: NSignal<[f32; 2]>,
        size: NSignal<[f32; 2]>,
        drop: NSignal<()>,
    }

    #[derive(Default)]
    pub struct VideoNode {
        videos: Vec<RVideo>,
        shader: Option<Shader>,

        pending: Option<RVideoInner>,
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
            let (sdrop, drop) = create_signal();

            self.pending = Some(RVideoInner {
                position,
                size,
                drop,
            });
            Box::new((sposition, ssize, sdrop))
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
    use std::{any::Any, mem::MaybeUninit, path::Path, sync::Arc};

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
        fn index(&self) -> usize;
        fn clone_ref(&self) -> Box<dyn Stream>;

        fn send_packet(&self, decoder: &mut Box<dyn Any>, packet: Packet);

        fn next(&self) -> bool;
        fn prev(&self) -> bool;
        fn clear(&self);

        fn data(&self, index: usize) -> Option<&[u8]>;
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
        frames: Vec<VFrame>,
        current: usize,

        index: usize,
    }

    impl VideoStream {
        pub fn new(index: usize) -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self {
                frames: Vec::default(),
                current: usize::MAX,
                index,
            }))
        }
    }

    impl Stream for Arc<RwLock<VideoStream>> {
        fn ty(&self) -> StreamType {
            StreamType::Video
        }

        fn index(&self) -> usize {
            self.blocking_read().index
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

            if !s.frames.is_empty() && s.current == usize::MAX {
                s.current = 0;
                return true;
            }

            println!("Current: {}, Frames: {}", s.current, s.frames.len());
            if s.current < s.frames.len() {
                s.current += 1;
                true
            } else {
                false
            }
        }

        fn prev(&self) -> bool {
            let s = &mut *self.try_write().unwrap();

            if s.current > 0 {
                s.current -= 1;
                true
            } else {
                false
            }
        }

        fn gc(&self) {
            let s = &mut *self.try_write().unwrap();

            if s.current > 100 && s.current != usize::MAX {
                s.frames.drain(..50);
                s.current -= 50;
            }
        }

        fn clear(&self) {
            let s = &mut *self.try_write().unwrap();
            s.current = usize::MAX;
            s.frames.clear();
        }

        fn data(&self, index: usize) -> Option<&[u8]> {
            let s = &*self.try_read().unwrap();

            if s.current == usize::MAX {
                return None;
            }

            let f = &s.frames[s.current];
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
            if s.current == usize::MAX {
                return None;
            }
            Some(s.frames[s.current].width())
        }

        fn height(&self) -> Option<u32> {
            let s = &*self.try_read().unwrap();
            if s.current == usize::MAX {
                return None;
            }
            Some(s.frames[s.current].height())
        }

        fn current(&self) -> usize {
            self.try_read().unwrap().current
        }
    }

    pub struct AudioStream {
        frames: Vec<AFrame>,
        current: usize,

        index: usize,
    }

    impl AudioStream {
        pub fn new(index: usize) -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self {
                frames: Vec::default(),
                current: usize::MAX,
                index,
            }))
        }
    }

    impl Stream for Arc<RwLock<AudioStream>> {
        fn ty(&self) -> StreamType {
            StreamType::Audio
        }

        fn index(&self) -> usize {
            self.try_read().unwrap().index
        }

        fn send_packet(&self, decoder: &mut Box<dyn Any>, packet: Packet) {
            let decoder = decoder.downcast_mut::<AudioDecoder>().unwrap();
            decoder.decoder.send_packet(&packet).unwrap();
            let mut frame = AFrame::empty();
            if decoder.decoder.receive_frame(&mut frame).is_err() {
                return;
            }

            let mut dst = AFrame::new(
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
                frame.samples(),
                ChannelLayout::STEREO,
            );
            let mut sws = frame
                .resampler(
                    ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
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

            if !s.frames.is_empty() && s.current == usize::MAX {
                s.current = 0;
                return true;
            }

            if s.current < s.frames.len() {
                s.current += 1;
                true
            } else {
                false
            }
        }

        fn prev(&self) -> bool {
            let s = &mut *self.try_write().unwrap();

            if s.current > 0 {
                s.current -= 1;
                true
            } else {
                false
            }
        }

        fn gc(&self) {
            let s = &mut *self.try_write().unwrap();

            if s.current > 100 && s.current != usize::MAX {
                s.frames.drain(..50);
                s.current -= 50;
            }
        }

        fn clear(&self) {
            let s = &mut *self.try_write().unwrap();
            s.current = usize::MAX;
            s.frames.clear();
        }

        fn data(&self, index: usize) -> Option<&[u8]> {
            let s = &*self.blocking_read();

            if s.current == usize::MAX {
                return None;
            }

            let f = &s.frames[s.current];
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

            if s.current == usize::MAX {
                return None;
            }

            let f = &s.frames[s.current];
            Some(f.samples())
        }

        fn channels(&self) -> Option<usize> {
            let s = &*self.blocking_read();

            if s.current == usize::MAX {
                return None;
            }

            let f = &s.frames[s.current];
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
                if !readys[i] {
                    readys[i] = self.streams[i].next()
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
        element::ElementBuilder,
        node::Node,
        signal::{create_signal, NSignal, Signal, SignalInner},
    };

    use crate::media::Stream;

    pub struct AudioBuilder {
        stream: Box<dyn Stream>,
    }

    impl AudioBuilder {
        pub fn new(stream: Box<dyn Stream>) -> Self {
            Self { stream }
        }
    }

    impl ElementBuilder for AudioBuilder {
        type Element<'a> = Audio<'a>;

        fn node_id(&self) -> std::any::TypeId {
            core::any::TypeId::of::<AudioNode>()
        }

        fn create_element_ref<'a>(
            &self,
            inner: Box<dyn std::any::Any + Send + Sync + 'static>,
            scene: &'a motion_man::scene::SceneTask,
        ) -> Self::Element<'a> {
            let drop = *inner.downcast::<SignalInner<()>>().unwrap();

            Audio {
                drop: Signal::new(drop, scene, ()),
                droped: false,
            }
        }
    }

    pub struct Audio<'a> {
        drop: Signal<'a, ()>,
        droped: bool,
    }

    impl<'a> Audio<'a> {
        pub async fn drop(mut self) {
            self.drop.set(()).await;
            self.droped = true;
        }
    }

    impl<'a> Drop for Audio<'a> {
        fn drop(&mut self) {
            if !self.droped {
                eprintln!("You need to call drop on Audio when is no more needed!");
                std::process::abort();
            }
        }
    }

    #[derive(Default)]
    pub struct AudioNode {
        audios: Vec<(NSignal<()>, Box<dyn Stream>, Vec<f32>, usize)>,
        pending: Option<NSignal<()>>,
    }

    impl Node for AudioNode {
        type ElementBuilder = AudioBuilder;

        fn init_element(&mut self, gcx: &motion_man::gcx::GCX, builder: Self::ElementBuilder) {
            let drop = self.pending.take().unwrap();
            self.audios
                .push((drop, builder.stream, Vec::new(), usize::MAX));
        }

        fn create_element(&mut self) -> Box<dyn std::any::Any + Send + Sync + 'static> {
            let (drop, ndrop) = create_signal::<()>();

            self.pending = Some(ndrop);

            Box::new(drop)
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
                if audio.1.current() != audio.3 {
                    if let Some(data) = audio.1.data(0) {
                        let samples = audio.1.samples().unwrap() * audio.1.channels().unwrap();
                        let buf: &[f32] = bytemuck::cast_slice(data);
                        audio.2.extend(&buf[..samples]);
                    }
                    audio.3 = audio.1.current();
                }

                let mut tmp = audio
                    .2
                    .drain(..buffer.len().min(audio.2.len()))
                    .collect::<Vec<f32>>();
                tmp.resize(buffer.len(), 0.);
                for i in 0..tmp.len() {
                    buffer[i] += tmp[i];
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    ffmpeg_next::init().unwrap();
    let version = ffmpeg_next::util::version();
    println!("FFMPEG: {version}");

    {
        // ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Trace);
        // ffmpeg_next::log::set_flags(ffmpeg_next::log::Flags::SKIP_REPEATED);
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

    let mut v = vec![0.; 48000 / 2];
    for i in 0..v.len() {
        v[i] = f32::sin(i as f32 * 0.1);
    }
    sender.send(v).unwrap();

    let mut v = vec![0.; 48000 / 2];
    for (i, v) in v.iter_mut().enumerate() {
        *v = f32::sin(i as f32 * 0.09);
    }
    sender.send(v).unwrap();

    let mut v = vec![0.; 48000 / 2];
    for (i, v) in v.iter_mut().enumerate() {
        *v = f32::sin(i as f32 * 0.08);
    }
    sender.send(v).unwrap();

    std::thread::sleep(Duration::from_secs_f32(2.0));

    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let _enter = rt.enter();

    let mut engine = Engine::new(60., 1920.try_into()?, 1080.try_into()?, 48000, 2);

    engine.register_node::<RectNode>();
    engine.register_node::<VideoNode>();
    engine.register_node::<AudioNode>();

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

            let mut media = Media::new("video.mkv").unwrap();
            let mut video = scene
                .spawn_element(VideoBuilder::new(media.video(0).unwrap()))
                .await;
            let audio = scene
                .spawn_element(AudioBuilder::new(media.audio(0).unwrap()))
                .await;

            video.size.tween([0., 0.], [1., 1.], 1.0).await;

            while media.next() {
                scene.wait(1).await;
            }

            video.size.tween([1., 1.], [0.1, 0.1], 1.0).await;

            audio.drop().await;
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
        let buffer = engine.audio_buffer();
        sender.send(buffer.to_vec()).unwrap();
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
