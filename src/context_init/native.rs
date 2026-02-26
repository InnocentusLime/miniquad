use std::num::NonZeroU32;

use crate::{AppEvent, Conf};

use glow::HasContext;
use glutin::config::{Api, Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version,
};
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::Window;

pub const GLSL_VERSION: &str = "#version 330 core";

pub fn start_app<T>(event_loop: EventLoop<T>, mut app: impl ApplicationHandler<T> + 'static) {
    event_loop.run_app(&mut app).expect("failed to run app");
}

pub fn create_ctx_and_window(
    event_loop: &ActiveEventLoop,
    _proxy: EventLoopProxy<AppEvent>,
    conf: &Conf,
) -> (Window, PlatformContext) {
    let (window, gl_display, gl_config) = create_window_and_gl_config(event_loop, conf);
    let (gl_context, gl_surface) =
        create_surface_and_context(conf.is_debug, &gl_display, &gl_config, &window);

    (
        window,
        PlatformContext {
            gl_display,
            gl_surface,
            gl_context,
        },
    )
}

pub struct PlatformContext {
    gl_display: Display,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
}

impl PlatformContext {
    pub fn resize_surface(&self, new_size: PhysicalSize<u32>) {
        // NOTE: winit may absolutely easily give us a new size equal to (0, 0).
        //       we can't do anything here, except pray that the user will eventually
        //       give us a proper size.
        let (Some(width), Some(height)) = (
            NonZeroU32::new(new_size.width),
            NonZeroU32::new(new_size.height),
        ) else {
            return;
        };
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn make_glow_context(&self, is_debug: bool) -> glow::Context {
        let mut ctx = unsafe {
            glow::Context::from_loader_function_cstr(|proc| self.gl_display.get_proc_address(proc))
        };
        if is_debug {
            unsafe {
                ctx.debug_message_callback(debug_message_callback);
                ctx.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
            }
        }
        ctx
    }

    pub fn swap_buffers(&self) {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .expect("Failed to swap buffers");
    }
}

fn create_window_and_gl_config(
    event_loop: &ActiveEventLoop,
    conf: &Conf,
) -> (Window, Display, Config) {
    let display_builder =
        DisplayBuilder::new().with_window_attributes(Some(conf.window_attributes.clone()));
    let template_builder = ConfigTemplateBuilder::new()
        .with_api(Api::OPENGL)
        .with_alpha_size(8);
    let (window, gl_config) = display_builder
        .build(event_loop, template_builder, |mut conf| {
            conf.next().expect("No GL configuration found")
        })
        .expect("Could not initialize the GL platform");
    let window = window.expect("No window has been created");

    (window, gl_config.display(), gl_config)
}

fn create_surface_and_context(
    is_debug: bool,
    gl_display: &Display,
    gl_config: &Config,
    window: &Window,
) -> (PossiblyCurrentContext, Surface<WindowSurface>) {
    let raw_window_handle = window
        .window_handle()
        .expect("Window has not raw handle")
        .as_raw();
    let context_attributes = ContextAttributesBuilder::new()
        .with_profile(GlProfile::Core)
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .with_debug(is_debug)
        .build(Some(raw_window_handle));
    let gl_context = unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .expect("Failed to create GL context")
    };
    let gl_context = gl_context.treat_as_possibly_current();

    let surface_attributes = window
        .build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");
    let surface = unsafe {
        gl_display
            .create_window_surface(gl_config, &surface_attributes)
            .expect("Failed to create window surface")
    };
    gl_context
        .make_current(&surface)
        .expect("Failed to make the context current");
    surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .expect("Failed to update window swap interval");

    (gl_context, surface)
}

fn debug_message_callback(source: u32, ty: u32, id: u32, severity: u32, msg: &str) {
    static DEBUG_MESSAGE: &str = "gl.debug";

    let src = match source {
        glow::DEBUG_SOURCE_API => "API",
        glow::DEBUG_SOURCE_APPLICATION => "app",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
        glow::DEBUG_SOURCE_THIRD_PARTY => "third party",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
        glow::DEBUG_SOURCE_OTHER => "other",
        _ => "N/A",
    };
    let ty_name = match ty {
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "depreacated",
        glow::DEBUG_TYPE_ERROR => "error",
        glow::DEBUG_TYPE_MARKER => "marker",
        glow::DEBUG_TYPE_OTHER => "other",
        glow::DEBUG_TYPE_PERFORMANCE => "performance",
        glow::DEBUG_TYPE_POP_GROUP => "pop group",
        glow::DEBUG_TYPE_PORTABILITY => "portability",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
        _ => "N/A",
    };

    match (ty, severity) {
        (glow::DEBUG_TYPE_ERROR, _) => tracing::error!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty_name,
            id=id,
            msg,
        ),
        (
            glow::DEBUG_TYPE_PERFORMANCE
            | glow::DEBUG_TYPE_PORTABILITY
            | glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
            _,
        ) => tracing::warn!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty,
            id=id,
            msg,
        ),
        (_, glow::DEBUG_SEVERITY_HIGH) => tracing::error!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty_name,
            id=id,
            msg,
        ),
        (_, glow::DEBUG_SEVERITY_MEDIUM) => tracing::warn!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty_name,
            id=id,
            msg,
        ),
        (_, glow::DEBUG_SEVERITY_LOW | glow::DEBUG_SEVERITY_NOTIFICATION) => tracing::info!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty_name,
            id=id,
            msg,
        ),
        _ => tracing::debug!(
            target:DEBUG_MESSAGE,
            src=src,
            ty=ty_name,
            id=id,
            msg,
        ),
    }
}
