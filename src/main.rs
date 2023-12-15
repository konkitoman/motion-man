use std::{
    borrow::Cow,
    error::Error,
    ffi::{CStr, CString},
    path::Path,
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
    gcx::{buffer::BufferUsage, BufferBit, GCX, GL},
    rect::{RectBuilder, RectNode},
};

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

use rusty_ffmpeg::ffi as FF;

#[derive(Debug, Clone, Copy)]
pub enum AVError {
    BsfNotFound,
    Bug,
    BufferTooSmall,
    DecoderNotFound,
    DemuxerNotFound,
    EncoderNotFound,
    Eof,
    Exit,
    External,
    FilterNotFound,
    Invaliddata,
    MuxerNotFound,
    OptionNotFound,
    Patchwelcome,
    ProtocolNotFound,

    StreamNotFound,

    Bug2,
    Unknown(core::ffi::c_int),

    HttpBadRequest,
    HttpUnauthorized,
    HttpForbidden,
    HttpNotFound,
    HttpOther4xx,
    HttpServerError,

    EPERM,
    ENOENT,
    ESRCH,
    EINTR,
    EIO,
    ENXIO,
    E2BIG,
    ENOEXEC,
    EBADF,
    ECHILD,
    EAGAIN,
    ENOMEM,
    EACCES,
    EFAULT,
    ENOTBLK,
    EBUSY,
    EEXIST,
    EXDEV,
    ENODEV,
    ENOTDIR,
    EISDIR,
    EINVAL,
    ENFILE,
    EMFILE,
    ENOTTY,
    ETXTBSY,
    EFBIG,
    ENOSPC,
    ESPIPE,
    EROFS,
    EMLINK,
    EPIPE,
    EDOM,
    ERANGE,
    EDEADLK,
    ENAMETOOLONG,
    ENOLCK,
    ENOSYS,
    ENOTEMPTY,
    ELOOP,
    EWOULDBLOCK,
    ENOMSG,
    EIDRM,
    ECHRNG,
    EL2NSYNC,
    EL3HLT,
    EL3RST,
    ELNRNG,
    EUNATCH,
    ENOCSI,
    EL2HLT,
    EBADE,
    EBADR,
    EXFULL,
    ENOANO,
    EBADRQC,
    EBADSLT,
    EDEADLOCK,
    EBFONT,
    ENOSTR,
    ENODATA,
    ETIME,
    ENOSR,
    ENONET,
    ENOPKG,
    EREMOTE,
    ENOLINK,
    EADV,
    ESRMNT,
    ECOMM,
    EPROTO,
    EMULTIHOP,
    EDOTDOT,
    EBADMSG,
    EOVERFLOW,
    ENOTUNIQ,
    EBADFD,
    EREMCHG,
    ELIBACC,
    ELIBBAD,
    ELIBSCN,
    ELIBMAX,
    ELIBEXEC,
    EILSEQ,
    ERESTART,
    ESTRPIPE,
    EUSERS,
    ENOTSOCK,
    EDESTADDRREQ,
    EMSGSIZE,
    EPROTOTYPE,
    ENOPROTOOPT,
    EPROTONOSUPPORT,
    ESOCKTNOSUPPORT,
    EOPNOTSUPP,
    EPFNOSUPPORT,
    EAFNOSUPPORT,
    EADDRINUSE,
    EADDRNOTAVAIL,
    ENETDOWN,
    ENETUNREACH,
    ENETRESET,
    ECONNABORTED,
    ECONNRESET,
    ENOBUFS,
    EISCONN,
    ENOTCONN,
    ESHUTDOWN,
    ETOOMANYREFS,
    ETIMEDOUT,
    ECONNREFUSED,
    EHOSTDOWN,
    EHOSTUNREACH,
    EALREADY,
    EINPROGRESS,
    ESTALE,
    EUCLEAN,
    ENOTNAM,
    ENAVAIL,
    EISNAM,
    EREMOTEIO,
    EDQUOT,
    ENOMEDIUM,
    EMEDIUMTYPE,
    ECANCELED,
    ENOKEY,
    EKEYEXPIRED,
    EKEYREVOKED,
    EKEYREJECTED,
    EOWNERDEAD,
    ENOTRECOVERABLE,
    ERFKILL,
    EHWPOISON,
    ENOTSUP,
}

impl From<core::ffi::c_uint> for AVError {
    fn from(value: core::ffi::c_uint) -> Self {
        match value {
            FF::EPERM => Self::EPERM,
            FF::ENOENT => Self::ENOENT,
            FF::ESRCH => Self::ESRCH,
            FF::EINTR => Self::EINTR,
            FF::EIO => Self::EIO,
            FF::ENXIO => Self::ENXIO,
            FF::E2BIG => Self::E2BIG,
            FF::ENOEXEC => Self::ENOEXEC,
            FF::EBADF => Self::EBADF,
            FF::ECHILD => Self::ECHILD,
            FF::EAGAIN => Self::EAGAIN,
            FF::ENOMEM => Self::ENOMEM,
            FF::EACCES => Self::EACCES,
            FF::EFAULT => Self::EFAULT,
            FF::ENOTBLK => Self::ENOTBLK,
            FF::EBUSY => Self::EBUSY,
            FF::EEXIST => Self::EEXIST,
            FF::EXDEV => Self::EXDEV,
            FF::ENODEV => Self::ENODEV,
            FF::ENOTDIR => Self::ENOTDIR,
            FF::EISDIR => Self::EISDIR,
            FF::EINVAL => Self::EINVAL,
            FF::ENFILE => Self::ENFILE,
            FF::EMFILE => Self::EMFILE,
            FF::ENOTTY => Self::ENOTTY,
            FF::ETXTBSY => Self::ETXTBSY,
            FF::EFBIG => Self::EFBIG,
            FF::ENOSPC => Self::ENOSPC,
            FF::ESPIPE => Self::ESPIPE,
            FF::EROFS => Self::EROFS,
            FF::EMLINK => Self::EMLINK,
            FF::EPIPE => Self::EPIPE,
            FF::EDOM => Self::EDOM,
            FF::ERANGE => Self::ERANGE,
            FF::EDEADLK => Self::EDEADLK,
            FF::ENAMETOOLONG => Self::ENAMETOOLONG,
            FF::ENOLCK => Self::ENOLCK,
            FF::ENOSYS => Self::ENOSYS,
            FF::ENOTEMPTY => Self::ENOTEMPTY,
            FF::ELOOP => Self::ELOOP,
            FF::EWOULDBLOCK => Self::EWOULDBLOCK,
            FF::ENOMSG => Self::ENOMSG,
            FF::EIDRM => Self::EIDRM,
            FF::ECHRNG => Self::ECHRNG,
            FF::EL2NSYNC => Self::EL2NSYNC,
            FF::EL3HLT => Self::EL3HLT,
            FF::EL3RST => Self::EL3RST,
            FF::ELNRNG => Self::ELNRNG,
            FF::EUNATCH => Self::EUNATCH,
            FF::ENOCSI => Self::ENOCSI,
            FF::EL2HLT => Self::EL2HLT,
            FF::EBADE => Self::EBADE,
            FF::EBADR => Self::EBADR,
            FF::EXFULL => Self::EXFULL,
            FF::ENOANO => Self::ENOANO,
            FF::EBADRQC => Self::EBADRQC,
            FF::EBADSLT => Self::EBADSLT,
            FF::EDEADLOCK => Self::EDEADLOCK,
            FF::EBFONT => Self::EBFONT,
            FF::ENOSTR => Self::ENOSTR,
            FF::ENODATA => Self::ENODATA,
            FF::ETIME => Self::ETIME,
            FF::ENOSR => Self::ENOSR,
            FF::ENONET => Self::ENONET,
            FF::ENOPKG => Self::ENOPKG,
            FF::EREMOTE => Self::EREMOTE,
            FF::ENOLINK => Self::ENOLINK,
            FF::EADV => Self::EADV,
            FF::ESRMNT => Self::ESRMNT,
            FF::ECOMM => Self::ECOMM,
            FF::EPROTO => Self::EPROTO,
            FF::EMULTIHOP => Self::EMULTIHOP,
            FF::EDOTDOT => Self::EDOTDOT,
            FF::EBADMSG => Self::EBADMSG,
            FF::EOVERFLOW => Self::EOVERFLOW,
            FF::ENOTUNIQ => Self::ENOTUNIQ,
            FF::EBADFD => Self::EBADFD,
            FF::EREMCHG => Self::EREMCHG,
            FF::ELIBACC => Self::ELIBACC,
            FF::ELIBBAD => Self::ELIBBAD,
            FF::ELIBSCN => Self::ELIBSCN,
            FF::ELIBMAX => Self::ELIBMAX,
            FF::ELIBEXEC => Self::ELIBEXEC,
            FF::EILSEQ => Self::EILSEQ,
            FF::ERESTART => Self::ERESTART,
            FF::ESTRPIPE => Self::ESTRPIPE,
            FF::EUSERS => Self::EUSERS,
            FF::ENOTSOCK => Self::ENOTSOCK,
            FF::EDESTADDRREQ => Self::EDESTADDRREQ,
            FF::EMSGSIZE => Self::EMSGSIZE,
            FF::EPROTOTYPE => Self::EPROTOTYPE,
            FF::ENOPROTOOPT => Self::ENOPROTOOPT,
            FF::EPROTONOSUPPORT => Self::EPROTONOSUPPORT,
            FF::ESOCKTNOSUPPORT => Self::ESOCKTNOSUPPORT,
            FF::EOPNOTSUPP => Self::EOPNOTSUPP,
            FF::EPFNOSUPPORT => Self::EPFNOSUPPORT,
            FF::EAFNOSUPPORT => Self::EAFNOSUPPORT,
            FF::EADDRINUSE => Self::EADDRINUSE,
            FF::EADDRNOTAVAIL => Self::EADDRNOTAVAIL,
            FF::ENETDOWN => Self::ENETDOWN,
            FF::ENETUNREACH => Self::ENETUNREACH,
            FF::ENETRESET => Self::ENETRESET,
            FF::ECONNABORTED => Self::ECONNABORTED,
            FF::ECONNRESET => Self::ECONNRESET,
            FF::ENOBUFS => Self::ENOBUFS,
            FF::EISCONN => Self::EISCONN,
            FF::ENOTCONN => Self::ENOTCONN,
            FF::ESHUTDOWN => Self::ESHUTDOWN,
            FF::ETOOMANYREFS => Self::ETOOMANYREFS,
            FF::ETIMEDOUT => Self::ETIMEDOUT,
            FF::ECONNREFUSED => Self::ECONNREFUSED,
            FF::EHOSTDOWN => Self::EHOSTDOWN,
            FF::EHOSTUNREACH => Self::EHOSTUNREACH,
            FF::EALREADY => Self::EALREADY,
            FF::EINPROGRESS => Self::EINPROGRESS,
            FF::ESTALE => Self::ESTALE,
            FF::EUCLEAN => Self::EUCLEAN,
            FF::ENOTNAM => Self::ENOTNAM,
            FF::ENAVAIL => Self::ENAVAIL,
            FF::EISNAM => Self::EISNAM,
            FF::EREMOTEIO => Self::EREMOTEIO,
            FF::EDQUOT => Self::EDQUOT,
            FF::ENOMEDIUM => Self::ENOMEDIUM,
            FF::EMEDIUMTYPE => Self::EMEDIUMTYPE,
            FF::ECANCELED => Self::ECANCELED,
            FF::ENOKEY => Self::ENOKEY,
            FF::EKEYEXPIRED => Self::EKEYEXPIRED,
            FF::EKEYREVOKED => Self::EKEYREVOKED,
            FF::EKEYREJECTED => Self::EKEYREJECTED,
            FF::EOWNERDEAD => Self::EOWNERDEAD,
            FF::ENOTRECOVERABLE => Self::ENOTRECOVERABLE,
            FF::ERFKILL => Self::ERFKILL,
            FF::EHWPOISON => Self::EHWPOISON,
            FF::ENOTSUP => Self::ENOTSUP,
            _ => Self::Unknown(value as i32),
        }
    }
}

impl From<core::ffi::c_int> for AVError {
    fn from(value: core::ffi::c_int) -> Self {
        match value {
            FF::AVERROR_BSF_NOT_FOUND => Self::BsfNotFound,
            FF::AVERROR_BUG => Self::Bug,
            FF::AVERROR_BUFFER_TOO_SMALL => Self::BufferTooSmall,
            FF::AVERROR_DECODER_NOT_FOUND => Self::DecoderNotFound,
            FF::AVERROR_DEMUXER_NOT_FOUND => Self::DemuxerNotFound,
            FF::AVERROR_ENCODER_NOT_FOUND => Self::EncoderNotFound,
            FF::AVERROR_EOF => Self::Eof,
            FF::AVERROR_EXIT => Self::Exit,
            FF::AVERROR_EXTERNAL => Self::External,
            FF::AVERROR_FILTER_NOT_FOUND => Self::FilterNotFound,
            FF::AVERROR_INVALIDDATA => Self::Invaliddata,
            FF::AVERROR_MUXER_NOT_FOUND => Self::MuxerNotFound,
            FF::AVERROR_OPTION_NOT_FOUND => Self::OptionNotFound,
            FF::AVERROR_PATCHWELCOME => Self::Patchwelcome,
            FF::AVERROR_PROTOCOL_NOT_FOUND => Self::ProtocolNotFound,

            FF::AVERROR_STREAM_NOT_FOUND => Self::StreamNotFound,

            FF::AVERROR_BUG2 => Self::Bug2,
            FF::AVERROR_UNKNOWN => Self::Unknown(0),

            FF::AVERROR_HTTP_BAD_REQUEST => Self::HttpBadRequest,
            FF::AVERROR_HTTP_UNAUTHORIZED => Self::HttpUnauthorized,
            FF::AVERROR_HTTP_FORBIDDEN => Self::HttpForbidden,
            FF::AVERROR_HTTP_NOT_FOUND => Self::HttpNotFound,
            FF::AVERROR_HTTP_OTHER_4XX => Self::HttpOther4xx,
            FF::AVERROR_HTTP_SERVER_ERROR => Self::HttpServerError,

            _ => Self::from(-value as core::ffi::c_uint),
        }
    }
}

pub struct AVFormatContext {
    row: *mut FF::AVFormatContext,
    url: CString,
}

impl AVFormatContext {
    pub fn new(url: impl Into<CString>) -> Result<Self, AVError> {
        let url = url.into();
        let mut row = unsafe { FF::avformat_alloc_context() };

        if row.is_null() {
            panic!("No memory!");
        }

        let err;

        unsafe {
            err = FF::avformat_open_input(
                &mut row,
                url.as_ptr(),
                core::ptr::null(),
                core::ptr::null_mut(),
            );
        }

        if err != 0 {
            unsafe { FF::avformat_free_context(row) };
            return Err(AVError::from(err));
        }

        Ok(Self { row, url })
    }

    pub fn read_frame(&mut self, packet: &mut AVPacket) -> Result<(), AVError> {
        let res = unsafe { FF::av_read_frame(self.row, packet.row) };

        if res != 0 {
            return Err(AVError::from(res));
        }

        Ok(())
    }

    pub fn streams_len(&self) -> u32 {
        unsafe { (*self.row).nb_streams }
    }

    pub fn streams(&mut self) -> impl Iterator<Item = AVStream> + '_ {
        let mut i = 0;
        core::iter::from_fn(move || unsafe {
            if (*self.row).nb_streams > i {
                let stream = *(*self.row).streams.offset(i as isize);
                i += 1;
                Some(AVStream { stream })
            } else {
                None
            }
        })
    }
}

impl Drop for AVFormatContext {
    fn drop(&mut self) {
        unsafe {
            FF::avformat_close_input(&mut self.row);
            FF::avformat_free_context(self.row);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum AVCodecType {
    Unknown,
    Video,
    Audio,
    Data,
    Subtitle,
    Attachment,
    Nb,
}

impl From<core::ffi::c_int> for AVCodecType {
    fn from(value: core::ffi::c_int) -> Self {
        match value {
            0 => Self::Video,
            1 => Self::Audio,
            2 => Self::Data,
            3 => Self::Subtitle,
            4 => Self::Attachment,
            5 => Self::Nb,
            _ => Self::Unknown,
        }
    }
}

pub struct AVStream {
    stream: *mut FF::AVStream,
}

impl AVStream {
    pub fn codec_params(&self) -> AVCodecParameters {
        let row = unsafe { (*self.stream).codecpar };
        if row.is_null() {
            panic!("No codec params");
        }

        AVCodecParameters { row }
    }

    pub fn codec_type(&self) -> AVCodecType {
        self.codec_params().ty()
    }

    pub fn decoder(&self) -> Option<AVCodec> {
        self.codec_params().find_decoder()
    }

    pub fn encoder(&self) -> Option<AVCodec> {
        self.codec_params().find_encoder()
    }
}

pub struct AVCodecParameters {
    row: *mut FF::AVCodecParameters,
}

impl AVCodecParameters {
    pub fn ty(&self) -> AVCodecType {
        AVCodecType::from(unsafe { (*self.row).codec_type })
    }

    pub fn find_decoder(&self) -> Option<AVCodec> {
        let row = unsafe { FF::avcodec_find_decoder((*self.row).codec_id) };

        if row.is_null() {
            return None;
        }

        Some(AVCodec { row })
    }

    pub fn find_encoder(&self) -> Option<AVCodec> {
        let row = unsafe { FF::avcodec_find_encoder((*self.row).codec_id) };

        if row.is_null() {
            return None;
        }

        Some(AVCodec { row })
    }
}

pub struct AVCodec {
    row: *const FF::AVCodec,
}

impl AVCodec {
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self.row).name) }
    }

    pub fn long_name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self.row).long_name) }
    }
}

pub struct AVCodecContext {
    row: *mut FF::AVCodecContext,
}

impl AVCodecContext {
    pub fn with_params(codec: &AVCodec, parameters: &AVCodecParameters) -> Result<Self, AVError> {
        let mut row = unsafe { FF::avcodec_alloc_context3(codec.row) };

        if row.is_null() {
            panic!("Error on avcodec_alloc_context3, possibile low memmory!");
        }

        let mut res;

        res = unsafe { FF::avcodec_parameters_to_context(row, parameters.row) };

        if res != 0 {
            unsafe { FF::avcodec_free_context(&mut row) };
            return Err(AVError::from(res));
        }

        res = unsafe { FF::avcodec_open2(row, codec.row, core::ptr::null_mut()) };

        if res != 0 {
            unsafe { FF::avcodec_free_context(&mut row) };
            return Err(AVError::from(res));
        }

        Ok(Self { row })
    }

    pub fn parameters_from_context(&self, parameters: &mut AVCodecParameters) {
        unsafe { FF::avcodec_parameters_from_context(parameters.row, self.row) };
    }

    pub fn send_packet(&mut self, packet: &AVPacket) -> Result<(), AVError> {
        let res = unsafe { FF::avcodec_send_packet(self.row, packet.row) };

        if res != 0 {
            return Err(AVError::from(res));
        }

        Ok(())
    }

    pub fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<(), AVError> {
        let res = unsafe { FF::avcodec_receive_frame(self.row, frame.row) };

        if res != 0 {
            return Err(AVError::from(res));
        }

        Ok(())
    }
}

impl Drop for AVCodecContext {
    fn drop(&mut self) {
        unsafe {
            FF::avcodec_close(self.row);
            FF::avcodec_free_context(&mut self.row);
        }
    }
}

pub struct AVFrame {
    row: *mut FF::AVFrame,
}

impl Default for AVFrame {
    fn default() -> Self {
        let row = unsafe { FF::av_frame_alloc() };

        if row.is_null() {
            panic!("Error on av_frame_alloc, possibile low memory!");
        }

        Self { row }
    }
}

impl Drop for AVFrame {
    fn drop(&mut self) {
        unsafe { FF::av_frame_free(&mut self.row) }
    }
}

pub struct AVPacket {
    row: *mut FF::AVPacket,
}

impl AVPacket {
    pub fn stream_index(&self) -> i32 {
        unsafe { (*self.row).stream_index }
    }
}

impl Default for AVPacket {
    fn default() -> Self {
        let row = unsafe { FF::av_packet_alloc() };

        if row.is_null() {
            panic!("Error on av_packet_alloc, possibile low memory!");
        }

        Self { row }
    }
}

impl Drop for AVPacket {
    fn drop(&mut self) {
        unsafe { FF::av_packet_free(&mut self.row) }
    }
}

mod video {
    use motion_man::{
        gcx::{
            buffer::BufferType,
            shader::{Shader, ShaderBuilder},
            vertex_array::VertexArray,
        },
        node::Node,
    };

    pub struct VideoNode {
        videos: Vec<VertexArray>,
        shader: Option<Shader>,
    }

    pub struct VideoBuilder {
        url: String,
        size: [f32; 2],
        pos: [f32; 2],
    }

    pub struct Video {
        id: usize,
    }

    impl Node for VideoNode {
        type ElementBuilder = VideoBuilder;

        fn init(&mut self, gcx: &motion_man::gcx::GCX) {
            self.shader.replace(
                ShaderBuilder::new()
                    .vertex(
                        r#"#version 320 es
                precision highp float;

                in vec4 pos;
                out vec2 UV;

                void main(){
                    gl_Position = vec4(pos.xy, 0.0, 1.0);
                    UV = pos.zw;
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
                    .build(&gcx)
                    .unwrap(),
            );
        }

        fn init_element(&mut self, gcx: &motion_man::gcx::GCX, builder: Self::ElementBuilder) {
            // let buffer = gcx.create_buffer(BufferType::ArrayBuffer, [], )
            // let va = gcx.create_vertex_array()
        }

        fn create_ref(&mut self) -> tokio::sync::mpsc::Sender<motion_man::element::ElementMessage> {
            todo!()
        }

        fn update(&mut self) {
            todo!()
        }

        fn render(&self, gcx: &motion_man::gcx::GCX) {
            todo!()
        }
    }

    fn create_mesh() {}
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
                .spawn_element(RectBuilder::new([1., 1.], Color::RED))
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

    let texture = unsafe { gcx.gl.create_texture().unwrap() };

    let v_buffer = gcx.create_buffer(
        motion_man::gcx::buffer::BufferType::ArrayBuffer,
        &[
            -1.0f32, -1.0, 0., 1., -1.0, 1.0, 0., 0., 1.0, 1.0, 1., 0., 1.0, -1.0, 1., 1.,
        ],
        BufferUsage::DRAW_STATIC,
    );
    let vao = gcx.create_vertex_array::<[f32; 4]>(v_buffer).build(&gcx);

    let shader = gcx
        .create_shader()
        .vertex(
            r#"#version 320 es
                precision highp float;

                in vec4 pos;
                out vec2 UV;

                void main(){
                    gl_Position = vec4(pos.xy, 0.0, 1.0);
                    UV = pos.zw;
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
        .build(&gcx)
        .unwrap();

    let version = unsafe { FF::avformat_version() };
    println!("AVFormat Version: {version}");

    let mut format_ctx = AVFormatContext::new(CString::new("video.mkv").unwrap()).unwrap();

    let mut streams = format_ctx.streams();
    let stream_1 = streams.next().unwrap();
    let stream_2 = streams.next().unwrap();

    drop(streams);

    println!("1: {:?}", stream_1.codec_type());

    let mut frame = AVFrame::default();
    let mut packet = AVPacket::default();
    let mut ctx = None;

    if let Some(codec) = stream_1.decoder() {
        println!("Decoder 1: {:?}", codec.long_name());
        ctx = Some(AVCodecContext::with_params(&codec, &stream_1.codec_params()).unwrap());
    }

    let mut ctx = ctx.unwrap();
    let dst;

    unsafe {
        dst = FF::av_frame_alloc();

        let res = FF::av_image_alloc(
            &mut (*dst).data as _,
            &mut (*dst).linesize as _,
            1920,
            1080,
            FF::AVPixelFormat_AV_PIX_FMT_RGB24,
            1,
        );

        if res < 0 {
            panic!("{:?}", AVError::from(res));
        }

        (*dst).width = 1920;
        (*dst).height = 1080;
        (*dst).format = FF::AVPixelFormat_AV_PIX_FMT_RGB24;
    }

    loop {
        let start = Instant::now();
        loop {
            if let Ok(()) = format_ctx.read_frame(&mut packet) {
                let pos = unsafe { (*packet.row).size };
                if packet.stream_index() == 0 {
                    ctx.send_packet(&packet);
                    if let Ok(()) = ctx.receive_frame(&mut frame) {
                        break;
                    }
                }
            } else {
                return Ok(());
            }
        }

        unsafe {
            let row = frame.row;

            let sws = FF::sws_getContext(
                (*row).width,
                (*row).height,
                (*row).format,
                1920,
                1080,
                (*dst).format,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null(),
            );

            FF::sws_scale(
                sws,
                (*row).data.as_ptr() as *const *const u8,
                (*row).linesize.as_ptr(),
                0,
                1080,
                (*dst).data.as_mut_ptr(),
                (*dst).linesize.as_mut_ptr(),
            );
        }

        gcx.clear_color(0xff);
        gcx.clear(BufferBit::COLOR);

        unsafe {
            let width = (*frame.row).width;
            let height = (*frame.row).height;
            let format = (*frame.row).format;

            _ = window.request_inner_size(LogicalSize::new(width / 2, height / 2));
            surface.resize(
                &context,
                (width as u32 / 2).try_into().unwrap(),
                (height as u32 / 2).try_into().unwrap(),
            );

            let mut buffer = vec![0u32; width as usize * height as usize];

            let mut i = 0;
            loop {
                let data = (*dst).data;
                let r = *data[0].offset(i) as u32;
                let g = *data[0].offset(i + 1) as u32;
                let b = *data[0].offset(i + 2) as u32;

                buffer[i as usize / 3] = 0xff000000 + r + (g << 8) + (b << 16);

                i += 3;
                if i >= width as isize * height as isize * 3 {
                    break;
                }
            }

            gcx.viewport(0, 0, 1920 / 2, 1080 / 2);

            gcx.use_shader(&shader, |gcx| {
                gcx.use_vertex_array(&vao, |gcx| {
                    gcx.gl.bind_texture(GL::TEXTURE_2D, Some(texture));
                    gcx.gl.tex_image_2d(
                        GL::TEXTURE_2D,
                        0,
                        GL::RGBA as i32,
                        width,
                        height,
                        0,
                        GL::RGBA,
                        GL::UNSIGNED_BYTE,
                        Some(bytemuck::cast_slice(&buffer)),
                    );
                    gcx.gl.generate_mipmap(GL::TEXTURE_2D);
                    // gcx.gl.active_texture(GL::TEXTURE0);
                    let location = gcx
                        .gl
                        .get_uniform_location(shader.program, "IMAGE")
                        .unwrap();
                    gcx.gl.uniform_1_i32(Some(&location), 0);
                    gcx.draw_arrays(motion_man::gcx::PrimitiveType::TrianglesFan, 0, 4);
                });
            });
        }

        surface.swap_buffers(&context).unwrap();
        if let Some(wait) = Duration::from_secs_f64(1. / 60.).checked_sub(start.elapsed()) {
            std::thread::sleep(wait);
        } else {
            println!("Is behind: {}", start.elapsed().as_secs_f64());
        }
    }

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
