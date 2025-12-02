#![doc = include_str!("../README.md")]

mod embeded_assets;
mod fs;
mod graphics;

pub use bytemuck::offset_of;
pub use fs::*;
pub use graphics::*;

use std::{num::NonZeroU32, rc::Rc};

use glutin::config::{Api, Config, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Icon, Window, WindowAttributes};

pub fn run<T: EventHandler>(conf: Conf) {
    let event_loop = EventLoop::<FileReady>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<T> {
        fs_server: FsServer::start(proxy),
        conf,
        state: AppState::Boot,
    };

    event_loop.run_app(&mut app).unwrap();
}

struct App<T> {
    conf: Conf,
    fs_server: FsServer,
    state: AppState<T>,
}

impl<T: EventHandler> ApplicationHandler<FileReady> for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match &mut self.state {
            AppState::Boot => self.init(event_loop),
            AppState::Ready { .. } => unimplemented!("Restoring of applications is not supported"),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: FileReady) {
        let AppState::Ready { handler, .. } = &mut self.state else {
            return;
        };
        handler.file_ready(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match (&event, &self.state) {
            (WindowEvent::CloseRequested, _) => event_loop.exit(),
            (
                WindowEvent::Resized(new_size),
                AppState::Ready {
                    surface,
                    gl_context,
                    ..
                },
            ) => {
                gl_context.client_area_size.set((*new_size).into());
                surface.resize(
                    &gl_context.glutin_ctx,
                    NonZeroU32::new(new_size.width).unwrap(),
                    NonZeroU32::new(new_size.height).unwrap(),
                );
            }
            _ => (),
        }

        let AppState::Ready {
            window,
            handler,
            surface,
            gl_context,
        } = &mut self.state
        else {
            return;
        };
        let swap_buffers = matches!(event, WindowEvent::RedrawRequested);
        handler.window_event(event, window);
        if swap_buffers {
            surface
                .swap_buffers(&gl_context.glutin_ctx)
                .expect("Failed to swap buffers");
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let AppState::Ready {
            window, handler, ..
        } = &mut self.state
        else {
            return;
        };
        window.request_redraw();
        handler.update();
    }
}

impl<T: EventHandler> App<T> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display, gl_config) = create_window_and_gl_config(event_loop, &self.conf);
        let (gl_context, surface) =
            create_surface_and_context(&display, &gl_config, &window, &self.conf);

        gl_context.make_current(&surface).unwrap();
        let glow_gl = unsafe {
            glow::Context::from_loader_function_cstr(|proc| display.get_proc_address(proc))
        };
        let gl_context = Rc::new(GlContext::new(
            gl_context,
            glow_gl,
            window.inner_size().into(),
        ));

        let handler = T::init(gl_context.clone(), self.fs_server.get_handle());
        self.state = AppState::Ready {
            window,
            surface,
            gl_context,
            handler,
        }
    }
}

enum AppState<T> {
    Boot,
    Ready {
        window: Window,
        surface: Surface<WindowSurface>,
        gl_context: Rc<GlContext>,
        handler: T,
    },
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
    conf: &Conf,
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
        .set_swap_interval(&gl_context, conf.swap_interval)
        .expect("Failed to update window swap interval");

    (gl_context, surface)
}

/// Describes a hardware and platform-specific setup.
#[derive(Debug)]
pub struct Conf {
    pub window_attributes: WindowAttributes,

    /// Optional swap interval (vertical sync).
    ///
    /// Note that this is highly platform- and driver-dependent.
    /// There is no guarantee the FPS will match the specified `swap_interval`.
    /// In other words, `swap_interval` is only a hint to the GPU driver and
    /// not a reliable way to limit the game's FPS.
    pub swap_interval: SwapInterval,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            window_attributes: default_window_attributes(),
            swap_interval: SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        }
    }
}

pub fn default_window_attributes() -> WindowAttributes {
    let default_icon = Icon::from_rgba(
        embeded_assets::DEFAULT_ICON.to_vec(),
        embeded_assets::DEFAULT_ICON_WIDTH,
        embeded_assets::DEFAULT_ICON_HEIGHT,
    )
    .unwrap();

    WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_resizable(true)
        .with_title("Miniquad window")
        .with_window_icon(Some(default_icon))
}

/// A trait defining event callbacks.
pub trait EventHandler {
    fn init(ctx: Rc<GlContext>, fs_server: FsServerHandle) -> Self;

    fn file_ready(&mut self, _event: FileReady) {}

    /// On most platforms update() and draw() are called each frame, sequentially,
    /// draw right after update.
    /// But on Android (and maybe some other platforms in the future) update might
    /// be called without draw.
    /// When the app is in background, Android destroys the rendering surface,
    /// while app is still alive and can do some usefull calculations.
    /// Note that in this case drawing from update may lead to crashes.
    fn update(&mut self);

    fn window_event(&mut self, event: WindowEvent, window: &Window);
}
