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
