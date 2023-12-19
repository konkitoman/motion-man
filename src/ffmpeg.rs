use std::ffi::{CStr, CString};

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
    pub fn new(url: impl Into<String>) -> Result<Self, AVError> {
        let url = url.into();
        let url = CString::new(url).expect("Invalid AVFormatContext url!");
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

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum AVPixelFormat {
    NONE = FF::AVPixelFormat_AV_PIX_FMT_NONE,
    YUV420P = FF::AVPixelFormat_AV_PIX_FMT_YUV420P,
    YUYV422 = FF::AVPixelFormat_AV_PIX_FMT_YUYV422,
    RGB24 = FF::AVPixelFormat_AV_PIX_FMT_RGB24,
    BGR24 = FF::AVPixelFormat_AV_PIX_FMT_BGR24,
    YUV422P = FF::AVPixelFormat_AV_PIX_FMT_YUV422P,
    YUV444P = FF::AVPixelFormat_AV_PIX_FMT_YUV444P,
    YUV410P = FF::AVPixelFormat_AV_PIX_FMT_YUV410P,
    YUV411P = FF::AVPixelFormat_AV_PIX_FMT_YUV411P,
    GRAY8 = FF::AVPixelFormat_AV_PIX_FMT_GRAY8,
    MONOWHITE = FF::AVPixelFormat_AV_PIX_FMT_MONOWHITE,
    MONOBLACK = FF::AVPixelFormat_AV_PIX_FMT_MONOBLACK,
    PAL8 = FF::AVPixelFormat_AV_PIX_FMT_PAL8,
    YUVJ420P = FF::AVPixelFormat_AV_PIX_FMT_YUVJ420P,
    YUVJ422P = FF::AVPixelFormat_AV_PIX_FMT_YUVJ422P,
    YUVJ444P = FF::AVPixelFormat_AV_PIX_FMT_YUVJ444P,
    UYVY422 = FF::AVPixelFormat_AV_PIX_FMT_UYVY422,
    UYYVYY411 = FF::AVPixelFormat_AV_PIX_FMT_UYYVYY411,
    BGR8 = FF::AVPixelFormat_AV_PIX_FMT_BGR8,
    BGR4 = FF::AVPixelFormat_AV_PIX_FMT_BGR4,
    BGR4Byte = FF::AVPixelFormat_AV_PIX_FMT_BGR4_BYTE,
    RGB8 = FF::AVPixelFormat_AV_PIX_FMT_RGB8,
    RGB4 = FF::AVPixelFormat_AV_PIX_FMT_RGB4,
    RGB4Byte = FF::AVPixelFormat_AV_PIX_FMT_RGB4_BYTE,
    NV12 = FF::AVPixelFormat_AV_PIX_FMT_NV12,
    NV21 = FF::AVPixelFormat_AV_PIX_FMT_NV21,
    ARGB = FF::AVPixelFormat_AV_PIX_FMT_ARGB,
    RGBA = FF::AVPixelFormat_AV_PIX_FMT_RGBA,
    ABGR = FF::AVPixelFormat_AV_PIX_FMT_ABGR,
    BGRA = FF::AVPixelFormat_AV_PIX_FMT_BGRA,
    GRAY16BE = FF::AVPixelFormat_AV_PIX_FMT_GRAY16BE,
    GRAY16LE = FF::AVPixelFormat_AV_PIX_FMT_GRAY16LE,
    YUV440P = FF::AVPixelFormat_AV_PIX_FMT_YUV440P,
    YUVJ440P = FF::AVPixelFormat_AV_PIX_FMT_YUVJ440P,
    YUVA420P = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P,
    RGB48BE = FF::AVPixelFormat_AV_PIX_FMT_RGB48BE,
    RGB48LE = FF::AVPixelFormat_AV_PIX_FMT_RGB48LE,
    RGB565BE = FF::AVPixelFormat_AV_PIX_FMT_RGB565BE,
    RGB565LE = FF::AVPixelFormat_AV_PIX_FMT_RGB565LE,
    RGB555BE = FF::AVPixelFormat_AV_PIX_FMT_RGB555BE,
    RGB555LE = FF::AVPixelFormat_AV_PIX_FMT_RGB555LE,
    BGR565BE = FF::AVPixelFormat_AV_PIX_FMT_BGR565BE,
    BGR565LE = FF::AVPixelFormat_AV_PIX_FMT_BGR565LE,
    BGR555BE = FF::AVPixelFormat_AV_PIX_FMT_BGR555BE,
    BGR555LE = FF::AVPixelFormat_AV_PIX_FMT_BGR555LE,
    VAAPI = FF::AVPixelFormat_AV_PIX_FMT_VAAPI,
    YUV420P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P16LE,
    YUV420P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P16BE,
    YUV422P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P16LE,
    YUV422P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P16BE,
    YUV444P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P16LE,
    YUV444P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P16BE,
    DXVA2_VLD = FF::AVPixelFormat_AV_PIX_FMT_DXVA2_VLD,
    RGB444LE = FF::AVPixelFormat_AV_PIX_FMT_RGB444LE,
    RGB444BE = FF::AVPixelFormat_AV_PIX_FMT_RGB444BE,
    BGR444LE = FF::AVPixelFormat_AV_PIX_FMT_BGR444LE,
    BGR444BE = FF::AVPixelFormat_AV_PIX_FMT_BGR444BE,
    // AVPixelFormat_AV_PIX_FMT_YA8 = FF::AVPixelFormat_AV_PIX_FMT_YA8,
    // AVPixelFormat_AV_PIX_FMT_Y400A = FF::AVPixelFormat_AV_PIX_FMT_Y400A,
    GRAY8A = FF::AVPixelFormat_AV_PIX_FMT_GRAY8A,
    BGR48BE = FF::AVPixelFormat_AV_PIX_FMT_BGR48BE,
    BGR48LE = FF::AVPixelFormat_AV_PIX_FMT_BGR48LE,
    YUV420P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P9BE,
    YUV420P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P9LE,
    YUV420P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P10BE,
    YUV420P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P10LE,
    YUV422P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P10BE,
    YUV422P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P10LE,
    YUV444P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P9BE,
    YUV444P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P9LE,
    YUV444P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P10BE,
    YUV444P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P10LE,
    YUV422P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P9BE,
    YUV422P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P9LE,
    GBRP = FF::AVPixelFormat_AV_PIX_FMT_GBRP,
    // AVPixelFormat_AV_PIX_FMT_GBR24P = FF::AVPixelFormat_AV_PIX_FMT_GBR24P,
    GBRP9BE = FF::AVPixelFormat_AV_PIX_FMT_GBRP9BE,
    GBRP9LE = FF::AVPixelFormat_AV_PIX_FMT_GBRP9LE,
    GBRP10BE = FF::AVPixelFormat_AV_PIX_FMT_GBRP10BE,
    GBRP10LE = FF::AVPixelFormat_AV_PIX_FMT_GBRP10LE,
    GBRP16BE = FF::AVPixelFormat_AV_PIX_FMT_GBRP16BE,
    GBRP16LE = FF::AVPixelFormat_AV_PIX_FMT_GBRP16LE,
    YUVA422P = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P,
    YUVA444P = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P,
    YUVA420P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P9BE,
    YUVA420P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P9LE,
    YUVA422P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P9BE,
    YUVA422P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P9LE,
    YUVA444P9BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P9BE,
    YUVA444P9LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P9LE,
    YUVA420P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P10BE,
    YUVA420P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P10LE,
    YUVA422P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P10BE,
    YUVA422P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P10LE,
    YUVA444P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P10BE,
    YUVA444P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P10LE,
    YUVA420P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P16BE,
    YUVA420P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA420P16LE,
    YUVA422P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P16BE,
    YUVA422P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P16LE,
    YUVA444P16BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P16BE,
    YUVA444P16LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P16LE,
    VDPAU = FF::AVPixelFormat_AV_PIX_FMT_VDPAU,
    XYZ12LE = FF::AVPixelFormat_AV_PIX_FMT_XYZ12LE,
    XYZ12BE = FF::AVPixelFormat_AV_PIX_FMT_XYZ12BE,
    NV16 = FF::AVPixelFormat_AV_PIX_FMT_NV16,
    NV20LE = FF::AVPixelFormat_AV_PIX_FMT_NV20LE,
    NV20BE = FF::AVPixelFormat_AV_PIX_FMT_NV20BE,
    RGBA64BE = FF::AVPixelFormat_AV_PIX_FMT_RGBA64BE,
    RGBA64LE = FF::AVPixelFormat_AV_PIX_FMT_RGBA64LE,
    BGRA64BE = FF::AVPixelFormat_AV_PIX_FMT_BGRA64BE,
    BGRA64LE = FF::AVPixelFormat_AV_PIX_FMT_BGRA64LE,
    YVYU422 = FF::AVPixelFormat_AV_PIX_FMT_YVYU422,
    YA16BE = FF::AVPixelFormat_AV_PIX_FMT_YA16BE,
    YA16LE = FF::AVPixelFormat_AV_PIX_FMT_YA16LE,
    GBRAP = FF::AVPixelFormat_AV_PIX_FMT_GBRAP,
    GBRAP16BE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP16BE,
    GBRAP16LE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP16LE,
    QSV = FF::AVPixelFormat_AV_PIX_FMT_QSV,
    MMAL = FF::AVPixelFormat_AV_PIX_FMT_MMAL,
    D3D11VA_VLD = FF::AVPixelFormat_AV_PIX_FMT_D3D11VA_VLD,
    CUDA = FF::AVPixelFormat_AV_PIX_FMT_CUDA,
    RGB = FF::AVPixelFormat_AV_PIX_FMT_0RGB,
    RGB0 = FF::AVPixelFormat_AV_PIX_FMT_RGB0,
    BGR = FF::AVPixelFormat_AV_PIX_FMT_0BGR,
    BGR0 = FF::AVPixelFormat_AV_PIX_FMT_BGR0,
    YUV420P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P12BE,
    YUV420P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P12LE,
    YUV420P14BE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P14BE,
    YUV420P14LE = FF::AVPixelFormat_AV_PIX_FMT_YUV420P14LE,
    YUV422P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P12BE,
    YUV422P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P12LE,
    YUV422P14BE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P14BE,
    YUV422P14LE = FF::AVPixelFormat_AV_PIX_FMT_YUV422P14LE,
    YUV444P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P12BE,
    YUV444P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P12LE,
    YUV444P14BE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P14BE,
    YUV444P14LE = FF::AVPixelFormat_AV_PIX_FMT_YUV444P14LE,
    GBRP12BE = FF::AVPixelFormat_AV_PIX_FMT_GBRP12BE,
    GBRP12LE = FF::AVPixelFormat_AV_PIX_FMT_GBRP12LE,
    GBRP14BE = FF::AVPixelFormat_AV_PIX_FMT_GBRP14BE,
    GBRP14LE = FF::AVPixelFormat_AV_PIX_FMT_GBRP14LE,
    YUVJ411P = FF::AVPixelFormat_AV_PIX_FMT_YUVJ411P,
    BAYER_BGGR8 = FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR8,
    BAYER_RGGB8 = FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB8,
    BAYER_GBRG8 = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG8,
    BAYER_GRBG8 = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG8,
    BAYER_BGGR16LE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR16LE,
    BAYER_BGGR16BE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR16BE,
    BAYER_RGGB16LE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB16LE,
    BAYER_RGGB16BE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB16BE,
    BAYER_GBRG16LE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG16LE,
    BAYER_GBRG16BE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG16BE,
    BAYER_GRBG16LE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16LE,
    BAYER_GRBG16BE = FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16BE,
    XVMC = FF::AVPixelFormat_AV_PIX_FMT_XVMC,
    YUV440P10LE = FF::AVPixelFormat_AV_PIX_FMT_YUV440P10LE,
    YUV440P10BE = FF::AVPixelFormat_AV_PIX_FMT_YUV440P10BE,
    YUV440P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUV440P12LE,
    YUV440P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUV440P12BE,
    AYUV64LE = FF::AVPixelFormat_AV_PIX_FMT_AYUV64LE,
    AYUV64BE = FF::AVPixelFormat_AV_PIX_FMT_AYUV64BE,
    VIDEOTOOLBOX = FF::AVPixelFormat_AV_PIX_FMT_VIDEOTOOLBOX,
    P010LE = FF::AVPixelFormat_AV_PIX_FMT_P010LE,
    P010BE = FF::AVPixelFormat_AV_PIX_FMT_P010BE,
    GBRAP12BE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP12BE,
    GBRAP12LE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP12LE,
    GBRAP10BE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP10BE,
    GBRAP10LE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP10LE,
    MEDIACODEC = FF::AVPixelFormat_AV_PIX_FMT_MEDIACODEC,
    GRAY12BE = FF::AVPixelFormat_AV_PIX_FMT_GRAY12BE,
    GRAY12LE = FF::AVPixelFormat_AV_PIX_FMT_GRAY12LE,
    GRAY10BE = FF::AVPixelFormat_AV_PIX_FMT_GRAY10BE,
    GRAY10LE = FF::AVPixelFormat_AV_PIX_FMT_GRAY10LE,
    P016LE = FF::AVPixelFormat_AV_PIX_FMT_P016LE,
    P016BE = FF::AVPixelFormat_AV_PIX_FMT_P016BE,
    D3D11 = FF::AVPixelFormat_AV_PIX_FMT_D3D11,
    GRAY9BE = FF::AVPixelFormat_AV_PIX_FMT_GRAY9BE,
    GRAY9LE = FF::AVPixelFormat_AV_PIX_FMT_GRAY9LE,
    GBRPF32BE = FF::AVPixelFormat_AV_PIX_FMT_GBRPF32BE,
    GBRPF32LE = FF::AVPixelFormat_AV_PIX_FMT_GBRPF32LE,
    GBRAPF32BE = FF::AVPixelFormat_AV_PIX_FMT_GBRAPF32BE,
    GBRAPF32LE = FF::AVPixelFormat_AV_PIX_FMT_GBRAPF32LE,
    DRM_PRIME = FF::AVPixelFormat_AV_PIX_FMT_DRM_PRIME,
    OPENCL = FF::AVPixelFormat_AV_PIX_FMT_OPENCL,
    GRAY14BE = FF::AVPixelFormat_AV_PIX_FMT_GRAY14BE,
    GRAY14LE = FF::AVPixelFormat_AV_PIX_FMT_GRAY14LE,
    GRAYF32BE = FF::AVPixelFormat_AV_PIX_FMT_GRAYF32BE,
    GRAYF32LE = FF::AVPixelFormat_AV_PIX_FMT_GRAYF32LE,
    YUVA422P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P12BE,
    YUVA422P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA422P12LE,
    YUVA444P12BE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P12BE,
    YUVA444P12LE = FF::AVPixelFormat_AV_PIX_FMT_YUVA444P12LE,
    NV24 = FF::AVPixelFormat_AV_PIX_FMT_NV24,
    NV42 = FF::AVPixelFormat_AV_PIX_FMT_NV42,
    VULKAN = FF::AVPixelFormat_AV_PIX_FMT_VULKAN,
    Y210BE = FF::AVPixelFormat_AV_PIX_FMT_Y210BE,
    Y210LE = FF::AVPixelFormat_AV_PIX_FMT_Y210LE,
    X2RGB10LE = FF::AVPixelFormat_AV_PIX_FMT_X2RGB10LE,
    X2RGB10BE = FF::AVPixelFormat_AV_PIX_FMT_X2RGB10BE,
    X2BGR10LE = FF::AVPixelFormat_AV_PIX_FMT_X2BGR10LE,
    X2BGR10BE = FF::AVPixelFormat_AV_PIX_FMT_X2BGR10BE,
    P210BE = FF::AVPixelFormat_AV_PIX_FMT_P210BE,
    P210LE = FF::AVPixelFormat_AV_PIX_FMT_P210LE,
    P410BE = FF::AVPixelFormat_AV_PIX_FMT_P410BE,
    P410LE = FF::AVPixelFormat_AV_PIX_FMT_P410LE,
    P216BE = FF::AVPixelFormat_AV_PIX_FMT_P216BE,
    P216LE = FF::AVPixelFormat_AV_PIX_FMT_P216LE,
    P416BE = FF::AVPixelFormat_AV_PIX_FMT_P416BE,
    P416LE = FF::AVPixelFormat_AV_PIX_FMT_P416LE,
    VUYA = FF::AVPixelFormat_AV_PIX_FMT_VUYA,
    RGBAF16BE = FF::AVPixelFormat_AV_PIX_FMT_RGBAF16BE,
    RGBAF16LE = FF::AVPixelFormat_AV_PIX_FMT_RGBAF16LE,
    VUYX = FF::AVPixelFormat_AV_PIX_FMT_VUYX,
    P012LE = FF::AVPixelFormat_AV_PIX_FMT_P012LE,
    P012BE = FF::AVPixelFormat_AV_PIX_FMT_P012BE,
    Y212BE = FF::AVPixelFormat_AV_PIX_FMT_Y212BE,
    Y212LE = FF::AVPixelFormat_AV_PIX_FMT_Y212LE,
    XV30BE = FF::AVPixelFormat_AV_PIX_FMT_XV30BE,
    XV30LE = FF::AVPixelFormat_AV_PIX_FMT_XV30LE,
    XV36BE = FF::AVPixelFormat_AV_PIX_FMT_XV36BE,
    XV36LE = FF::AVPixelFormat_AV_PIX_FMT_XV36LE,
    RGBF32BE = FF::AVPixelFormat_AV_PIX_FMT_RGBF32BE,
    RGBF32LE = FF::AVPixelFormat_AV_PIX_FMT_RGBF32LE,
    RGBAF32BE = FF::AVPixelFormat_AV_PIX_FMT_RGBAF32BE,
    RGBAF32LE = FF::AVPixelFormat_AV_PIX_FMT_RGBAF32LE,
    P212BE = FF::AVPixelFormat_AV_PIX_FMT_P212BE,
    P212LE = FF::AVPixelFormat_AV_PIX_FMT_P212LE,
    P412BE = FF::AVPixelFormat_AV_PIX_FMT_P412BE,
    P412LE = FF::AVPixelFormat_AV_PIX_FMT_P412LE,
    GBRAP14BE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP14BE,
    GBRAP14LE = FF::AVPixelFormat_AV_PIX_FMT_GBRAP14LE,
    NB = FF::AVPixelFormat_AV_PIX_FMT_NB,
}

impl From<i32> for AVPixelFormat {
    fn from(value: i32) -> Self {
        match value {
            FF::AVPixelFormat_AV_PIX_FMT_NONE => Self::NONE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P => Self::YUV420P,
            FF::AVPixelFormat_AV_PIX_FMT_YUYV422 => Self::YUYV422,
            FF::AVPixelFormat_AV_PIX_FMT_RGB24 => Self::RGB24,
            FF::AVPixelFormat_AV_PIX_FMT_BGR24 => Self::BGR24,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P => Self::YUV422P,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P => Self::YUV444P,
            FF::AVPixelFormat_AV_PIX_FMT_YUV410P => Self::YUV410P,
            FF::AVPixelFormat_AV_PIX_FMT_YUV411P => Self::YUV411P,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY8 => Self::GRAY8,
            FF::AVPixelFormat_AV_PIX_FMT_MONOWHITE => Self::MONOWHITE,
            FF::AVPixelFormat_AV_PIX_FMT_MONOBLACK => Self::MONOBLACK,
            FF::AVPixelFormat_AV_PIX_FMT_PAL8 => Self::PAL8,
            FF::AVPixelFormat_AV_PIX_FMT_YUVJ420P => Self::YUVJ420P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVJ422P => Self::YUVJ422P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVJ444P => Self::YUVJ444P,
            FF::AVPixelFormat_AV_PIX_FMT_UYVY422 => Self::UYVY422,
            FF::AVPixelFormat_AV_PIX_FMT_UYYVYY411 => Self::UYYVYY411,
            FF::AVPixelFormat_AV_PIX_FMT_BGR8 => Self::BGR8,
            FF::AVPixelFormat_AV_PIX_FMT_BGR4 => Self::BGR4,
            FF::AVPixelFormat_AV_PIX_FMT_BGR4_BYTE => Self::BGR4Byte,
            FF::AVPixelFormat_AV_PIX_FMT_RGB8 => Self::RGB8,
            FF::AVPixelFormat_AV_PIX_FMT_RGB4 => Self::RGB4,
            FF::AVPixelFormat_AV_PIX_FMT_RGB4_BYTE => Self::RGB4Byte,
            FF::AVPixelFormat_AV_PIX_FMT_NV12 => Self::NV12,
            FF::AVPixelFormat_AV_PIX_FMT_NV21 => Self::NV21,
            FF::AVPixelFormat_AV_PIX_FMT_ARGB => Self::ARGB,
            FF::AVPixelFormat_AV_PIX_FMT_RGBA => Self::RGBA,
            FF::AVPixelFormat_AV_PIX_FMT_ABGR => Self::ABGR,
            FF::AVPixelFormat_AV_PIX_FMT_BGRA => Self::BGRA,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY16BE => Self::GRAY16BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY16LE => Self::GRAY16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV440P => Self::YUV440P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVJ440P => Self::YUVJ440P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P => Self::YUVA420P,
            FF::AVPixelFormat_AV_PIX_FMT_RGB48BE => Self::RGB48BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB48LE => Self::RGB48LE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB565BE => Self::RGB565BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB565LE => Self::RGB565LE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB555BE => Self::RGB555BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB555LE => Self::RGB555LE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR565BE => Self::BGR565BE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR565LE => Self::BGR565LE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR555BE => Self::BGR555BE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR555LE => Self::BGR555LE,
            FF::AVPixelFormat_AV_PIX_FMT_VAAPI => Self::VAAPI,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P16LE => Self::YUV420P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P16BE => Self::YUV420P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P16LE => Self::YUV422P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P16BE => Self::YUV422P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P16LE => Self::YUV444P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P16BE => Self::YUV444P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_DXVA2_VLD => Self::DXVA2_VLD,
            FF::AVPixelFormat_AV_PIX_FMT_RGB444LE => Self::RGB444LE,
            FF::AVPixelFormat_AV_PIX_FMT_RGB444BE => Self::RGB444BE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR444LE => Self::BGR444LE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR444BE => Self::BGR444BE,
            // FF::AVPixelFormat_AV_PIX_FMT_YA8 => Self::AVPixelFormat_AV_PIX_FMT_YA8,
            // FF::AVPixelFormat_AV_PIX_FMT_Y400A => Self::AVPixelFormat_AV_PIX_FMT_Y400A,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY8A => Self::GRAY8A,
            FF::AVPixelFormat_AV_PIX_FMT_BGR48BE => Self::BGR48BE,
            FF::AVPixelFormat_AV_PIX_FMT_BGR48LE => Self::BGR48LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P9BE => Self::YUV420P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P9LE => Self::YUV420P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P10BE => Self::YUV420P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P10LE => Self::YUV420P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P10BE => Self::YUV422P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P10LE => Self::YUV422P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P9BE => Self::YUV444P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P9LE => Self::YUV444P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P10BE => Self::YUV444P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P10LE => Self::YUV444P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P9BE => Self::YUV422P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P9LE => Self::YUV422P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP => Self::GBRP,
            //FF:: AVPixelFormat_AV_PIX_FMT_GBR24P => Self::AVPixelFormat_AV_PIX_FMT_GBR24P,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP9BE => Self::GBRP9BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP9LE => Self::GBRP9LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP10BE => Self::GBRP10BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP10LE => Self::GBRP10LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP16BE => Self::GBRP16BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP16LE => Self::GBRP16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P => Self::YUVA422P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P => Self::YUVA444P,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P9BE => Self::YUVA420P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P9LE => Self::YUVA420P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P9BE => Self::YUVA422P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P9LE => Self::YUVA422P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P9BE => Self::YUVA444P9BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P9LE => Self::YUVA444P9LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P10BE => Self::YUVA420P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P10LE => Self::YUVA420P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P10BE => Self::YUVA422P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P10LE => Self::YUVA422P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P10BE => Self::YUVA444P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P10LE => Self::YUVA444P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P16BE => Self::YUVA420P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA420P16LE => Self::YUVA420P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P16BE => Self::YUVA422P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P16LE => Self::YUVA422P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P16BE => Self::YUVA444P16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P16LE => Self::YUVA444P16LE,
            FF::AVPixelFormat_AV_PIX_FMT_VDPAU => Self::VDPAU,
            FF::AVPixelFormat_AV_PIX_FMT_XYZ12LE => Self::XYZ12LE,
            FF::AVPixelFormat_AV_PIX_FMT_XYZ12BE => Self::XYZ12BE,
            FF::AVPixelFormat_AV_PIX_FMT_NV16 => Self::NV16,
            FF::AVPixelFormat_AV_PIX_FMT_NV20LE => Self::NV20LE,
            FF::AVPixelFormat_AV_PIX_FMT_NV20BE => Self::NV20BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBA64BE => Self::RGBA64BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBA64LE => Self::RGBA64LE,
            FF::AVPixelFormat_AV_PIX_FMT_BGRA64BE => Self::BGRA64BE,
            FF::AVPixelFormat_AV_PIX_FMT_BGRA64LE => Self::BGRA64LE,
            FF::AVPixelFormat_AV_PIX_FMT_YVYU422 => Self::YVYU422,
            FF::AVPixelFormat_AV_PIX_FMT_YA16BE => Self::YA16BE,
            FF::AVPixelFormat_AV_PIX_FMT_YA16LE => Self::YA16LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP => Self::GBRAP,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP16BE => Self::GBRAP16BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP16LE => Self::GBRAP16LE,
            FF::AVPixelFormat_AV_PIX_FMT_QSV => Self::QSV,
            FF::AVPixelFormat_AV_PIX_FMT_MMAL => Self::MMAL,
            FF::AVPixelFormat_AV_PIX_FMT_D3D11VA_VLD => Self::D3D11VA_VLD,
            FF::AVPixelFormat_AV_PIX_FMT_CUDA => Self::CUDA,
            FF::AVPixelFormat_AV_PIX_FMT_0RGB => Self::RGB,
            FF::AVPixelFormat_AV_PIX_FMT_RGB0 => Self::RGB0,
            FF::AVPixelFormat_AV_PIX_FMT_0BGR => Self::BGR,
            FF::AVPixelFormat_AV_PIX_FMT_BGR0 => Self::BGR0,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P12BE => Self::YUV420P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P12LE => Self::YUV420P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P14BE => Self::YUV420P14BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV420P14LE => Self::YUV420P14LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P12BE => Self::YUV422P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P12LE => Self::YUV422P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P14BE => Self::YUV422P14BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV422P14LE => Self::YUV422P14LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P12BE => Self::YUV444P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P12LE => Self::YUV444P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P14BE => Self::YUV444P14BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV444P14LE => Self::YUV444P14LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP12BE => Self::GBRP12BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP12LE => Self::GBRP12LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP14BE => Self::GBRP14BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRP14LE => Self::GBRP14LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVJ411P => Self::YUVJ411P,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR8 => Self::BAYER_BGGR8,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB8 => Self::BAYER_RGGB8,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG8 => Self::BAYER_GBRG8,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG8 => Self::BAYER_GRBG8,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR16LE => Self::BAYER_BGGR16LE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_BGGR16BE => Self::BAYER_BGGR16BE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB16LE => Self::BAYER_RGGB16LE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_RGGB16BE => Self::BAYER_RGGB16BE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG16LE => Self::BAYER_GBRG16LE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GBRG16BE => Self::BAYER_GBRG16BE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16LE => Self::BAYER_GRBG16LE,
            FF::AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16BE => Self::BAYER_GRBG16BE,
            FF::AVPixelFormat_AV_PIX_FMT_XVMC => Self::XVMC,
            FF::AVPixelFormat_AV_PIX_FMT_YUV440P10LE => Self::YUV440P10LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV440P10BE => Self::YUV440P10BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV440P12LE => Self::YUV440P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUV440P12BE => Self::YUV440P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_AYUV64LE => Self::AYUV64LE,
            FF::AVPixelFormat_AV_PIX_FMT_AYUV64BE => Self::AYUV64BE,
            FF::AVPixelFormat_AV_PIX_FMT_VIDEOTOOLBOX => Self::VIDEOTOOLBOX,
            FF::AVPixelFormat_AV_PIX_FMT_P010LE => Self::P010LE,
            FF::AVPixelFormat_AV_PIX_FMT_P010BE => Self::P010BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP12BE => Self::GBRAP12BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP12LE => Self::GBRAP12LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP10BE => Self::GBRAP10BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP10LE => Self::GBRAP10LE,
            FF::AVPixelFormat_AV_PIX_FMT_MEDIACODEC => Self::MEDIACODEC,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY12BE => Self::GRAY12BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY12LE => Self::GRAY12LE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY10BE => Self::GRAY10BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY10LE => Self::GRAY10LE,
            FF::AVPixelFormat_AV_PIX_FMT_P016LE => Self::P016LE,
            FF::AVPixelFormat_AV_PIX_FMT_P016BE => Self::P016BE,
            FF::AVPixelFormat_AV_PIX_FMT_D3D11 => Self::D3D11,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY9BE => Self::GRAY9BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY9LE => Self::GRAY9LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRPF32BE => Self::GBRPF32BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRPF32LE => Self::GBRPF32LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAPF32BE => Self::GBRAPF32BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAPF32LE => Self::GBRAPF32LE,
            FF::AVPixelFormat_AV_PIX_FMT_DRM_PRIME => Self::DRM_PRIME,
            FF::AVPixelFormat_AV_PIX_FMT_OPENCL => Self::OPENCL,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY14BE => Self::GRAY14BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAY14LE => Self::GRAY14LE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAYF32BE => Self::GRAYF32BE,
            FF::AVPixelFormat_AV_PIX_FMT_GRAYF32LE => Self::GRAYF32LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P12BE => Self::YUVA422P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA422P12LE => Self::YUVA422P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P12BE => Self::YUVA444P12BE,
            FF::AVPixelFormat_AV_PIX_FMT_YUVA444P12LE => Self::YUVA444P12LE,
            FF::AVPixelFormat_AV_PIX_FMT_NV24 => Self::NV24,
            FF::AVPixelFormat_AV_PIX_FMT_NV42 => Self::NV42,
            FF::AVPixelFormat_AV_PIX_FMT_VULKAN => Self::VULKAN,
            FF::AVPixelFormat_AV_PIX_FMT_Y210BE => Self::Y210BE,
            FF::AVPixelFormat_AV_PIX_FMT_Y210LE => Self::Y210LE,
            FF::AVPixelFormat_AV_PIX_FMT_X2RGB10LE => Self::X2RGB10LE,
            FF::AVPixelFormat_AV_PIX_FMT_X2RGB10BE => Self::X2RGB10BE,
            FF::AVPixelFormat_AV_PIX_FMT_X2BGR10LE => Self::X2BGR10LE,
            FF::AVPixelFormat_AV_PIX_FMT_X2BGR10BE => Self::X2BGR10BE,
            FF::AVPixelFormat_AV_PIX_FMT_P210BE => Self::P210BE,
            FF::AVPixelFormat_AV_PIX_FMT_P210LE => Self::P210LE,
            FF::AVPixelFormat_AV_PIX_FMT_P410BE => Self::P410BE,
            FF::AVPixelFormat_AV_PIX_FMT_P410LE => Self::P410LE,
            FF::AVPixelFormat_AV_PIX_FMT_P216BE => Self::P216BE,
            FF::AVPixelFormat_AV_PIX_FMT_P216LE => Self::P216LE,
            FF::AVPixelFormat_AV_PIX_FMT_P416BE => Self::P416BE,
            FF::AVPixelFormat_AV_PIX_FMT_P416LE => Self::P416LE,
            FF::AVPixelFormat_AV_PIX_FMT_VUYA => Self::VUYA,
            FF::AVPixelFormat_AV_PIX_FMT_RGBAF16BE => Self::RGBAF16BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBAF16LE => Self::RGBAF16LE,
            FF::AVPixelFormat_AV_PIX_FMT_VUYX => Self::VUYX,
            FF::AVPixelFormat_AV_PIX_FMT_P012LE => Self::P012LE,
            FF::AVPixelFormat_AV_PIX_FMT_P012BE => Self::P012BE,
            FF::AVPixelFormat_AV_PIX_FMT_Y212BE => Self::Y212BE,
            FF::AVPixelFormat_AV_PIX_FMT_Y212LE => Self::Y212LE,
            FF::AVPixelFormat_AV_PIX_FMT_XV30BE => Self::XV30BE,
            FF::AVPixelFormat_AV_PIX_FMT_XV30LE => Self::XV30LE,
            FF::AVPixelFormat_AV_PIX_FMT_XV36BE => Self::XV36BE,
            FF::AVPixelFormat_AV_PIX_FMT_XV36LE => Self::XV36LE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBF32BE => Self::RGBF32BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBF32LE => Self::RGBF32LE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBAF32BE => Self::RGBAF32BE,
            FF::AVPixelFormat_AV_PIX_FMT_RGBAF32LE => Self::RGBAF32LE,
            FF::AVPixelFormat_AV_PIX_FMT_P212BE => Self::P212BE,
            FF::AVPixelFormat_AV_PIX_FMT_P212LE => Self::P212LE,
            FF::AVPixelFormat_AV_PIX_FMT_P412BE => Self::P412BE,
            FF::AVPixelFormat_AV_PIX_FMT_P412LE => Self::P412LE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP14BE => Self::GBRAP14BE,
            FF::AVPixelFormat_AV_PIX_FMT_GBRAP14LE => Self::GBRAP14LE,
            FF::AVPixelFormat_AV_PIX_FMT_NB => Self::NB,
            _ => Self::NONE,
        }
    }
}

pub struct AVFrame {
    row: *mut FF::AVFrame,
    has_image: bool,
}

impl Default for AVFrame {
    fn default() -> Self {
        let row = unsafe { FF::av_frame_alloc() };

        if row.is_null() {
            panic!("Error on av_frame_alloc, possibile low memory!");
        }

        Self {
            row,
            has_image: false,
        }
    }
}

impl AVFrame {
    pub fn with_image(width: i32, height: i32, format: AVPixelFormat) -> Result<Self, AVError> {
        let mut row = unsafe { FF::av_frame_alloc() };

        if row.is_null() {
            panic!("Error on av_frame_alloc, possibile low memory!");
        }

        let res = unsafe {
            FF::av_image_alloc(
                &mut (*row).data as _,
                &mut (*row).linesize as _,
                width,
                height,
                format as i32,
                1,
            )
        };

        if res < 0 {
            unsafe { FF::av_frame_free(&mut row) };
            return Err(AVError::from(res));
        }

        unsafe {
            (*row).width = width;
            (*row).height = height;
            (*row).format = format as i32;
        }

        Ok(Self {
            row,
            has_image: true,
        })
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.row).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.row).height }
    }

    pub fn format(&self) -> AVPixelFormat {
        AVPixelFormat::from(unsafe { (*self.row).format })
    }

    pub fn data(&self) -> [&[u8]; 8] {
        unsafe {
            let row = self.row;
            let height = self.height() as usize;
            use core::slice::from_raw_parts as slice_from;
            [
                slice_from((*row).data[0], (*row).linesize[0] as usize * height),
                slice_from((*row).data[1], (*row).linesize[1] as usize * height),
                slice_from((*row).data[2], (*row).linesize[2] as usize * height),
                slice_from((*row).data[3], (*row).linesize[3] as usize * height),
                slice_from((*row).data[4], (*row).linesize[4] as usize * height),
                slice_from((*row).data[5], (*row).linesize[5] as usize * height),
                slice_from((*row).data[6], (*row).linesize[6] as usize * height),
                slice_from((*row).data[7], (*row).linesize[7] as usize * height),
            ]
        }
    }
}

impl Drop for AVFrame {
    fn drop(&mut self) {
        if self.has_image {
            unsafe { FF::av_freep(&mut (*self.row).data as *mut _ as *mut core::ffi::c_void) }
        }
        unsafe { FF::av_frame_free(&mut self.row) }
    }
}

pub struct SwsContext {
    row: *mut FF::SwsContext,
}

impl SwsContext {
    pub fn from_frame(frame: &AVFrame, dst: &AVFrame) -> Self {
        let row;
        unsafe {
            row = FF::sws_getContext(
                frame.width(),
                frame.height(),
                frame.format() as i32,
                dst.width(),
                dst.height(),
                dst.format() as i32,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null(),
            );
        }

        if row.is_null() {
            panic!("Error on sws_getContext");
        }

        Self { row }
    }

    pub fn sws_scale(&self, from: &AVFrame, to: &mut AVFrame) -> Result<(), AVError> {
        let res;
        unsafe {
            res = FF::sws_scale(
                self.row,
                (*from.row).data.as_ptr() as *const *const u8,
                (*from.row).linesize.as_ptr(),
                0,
                1080,
                (*to.row).data.as_mut_ptr(),
                (*to.row).linesize.as_mut_ptr(),
            );
        }

        if res < 0 {
            return Err(AVError::from(res));
        }

        Ok(())
    }
}

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe { FF::sws_freeContext(self.row) }
    }
}

pub struct AVPacket {
    row: *mut FF::AVPacket,
}

impl AVPacket {
    pub fn stream_index(&self) -> i32 {
        unsafe { (*self.row).stream_index }
    }

    pub fn size(&self) -> i32 {
        unsafe { (*self.row).size }
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
