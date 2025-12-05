use std::num::NonZeroU32;

use crate::Conf;

use glutin::config::{Api, Config, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

pub fn start_app<T>(event_loop: EventLoop<T>, mut app: impl ApplicationHandler<T> + 'static) {
    event_loop.run_app(&mut app).expect("failed to run app");
}

pub fn create_ctx_and_window(
    event_loop: &ActiveEventLoop,
    conf: &Conf,
) -> (Window, PlatformContext) {
    let (window, gl_display, gl_config) = create_window_and_gl_config(event_loop, conf);
    let (gl_context, gl_surface) = create_surface_and_context(&gl_display, &gl_config, &window);

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
        let (Some(width), Some(height)) = (NonZeroU32::new(new_size.width), NonZeroU32::new(new_size.height)) else {
            return;
        };
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn make_glow_context(&self) -> glow::Context {
        unsafe {
            glow::Context::from_loader_function_cstr(|proc| self.gl_display.get_proc_address(proc))
        }
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
    gl_display: &Display,
    gl_config: &Config,
    window: &Window,
) -> (PossiblyCurrentContext, Surface<WindowSurface>) {
    let raw_window_handle = window
        .window_handle()
        .expect("Window has not raw handle")
        .as_raw();
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
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
