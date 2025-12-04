#![doc = include_str!("../README.md")]

mod context_init;
mod embeded_assets;
mod fs;
mod graphics;
mod tracing_init;

pub mod util;

pub use bytemuck::offset_of;
pub use fs::*;
use glow::HasContext;
pub use graphics::*;
use tracing_subscriber::EnvFilter;
pub use web_time::*;

use std::path::PathBuf;
use std::rc::Rc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Icon, Window, WindowAttributes};

use crate::context_init::*;

static TARGET_NAME: &str = "app";

pub fn run<T: EventHandler>(conf: Conf) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let filter = EnvFilter::builder().parse_lossy("debug");
    tracing_init::init_tracing_subscriber(filter);
    tracing::info!(target: TARGET_NAME, conf=?conf, "starting");

    let event_loop = EventLoop::<FileReady>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    event_loop.set_control_flow(ControlFlow::Poll);

    start_app(
        event_loop,
        App::<T> {
            fs_server: FsServer::start(proxy, conf.fs_root.clone()),
            conf,
            state: AppState::Boot,
        },
    );
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
                    platform,
                    gl_context,
                    ..
                },
            ) => {
                tracing::debug!(
                    target: TARGET_NAME,
                    width=new_size.width,
                    height=new_size.height,
                    "resize",
                );
                platform.resize_surface(*new_size);
                gl_context.client_area_size.set((*new_size).into());
            }
            _ => (),
        }

        let AppState::Ready {
            window,
            handler,
            platform,
            gl_context,
            ..
        } = &mut self.state
        else {
            return;
        };
        let do_draw = matches!(event, WindowEvent::RedrawRequested);
        handler.window_event(event, window);
        if do_draw {
            unsafe {
                gl_context.gl.finish();
            }
            platform.swap_buffers();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let AppState::Ready {
            window, handler, ..
        } = &mut self.state
        else {
            return;
        };

        handler.update();

        window.request_redraw();
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now()
                .checked_add(Duration::from_millis(16))
                .unwrap(),
        ));
    }
}

impl<T: EventHandler> App<T> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let (window, platform) = create_ctx_and_window(event_loop, &self.conf);
        let gl_context = Rc::new(GlContext::new(platform.make_glow_context(), (800, 600)));
        tracing::info!(target: TARGET_NAME, "The context has been successfully created");

        let handler = T::init(gl_context.clone(), self.fs_server.get_handle());
        self.state = AppState::Ready {
            window,
            platform,
            gl_context,
            handler,
        }
    }
}

enum AppState<T> {
    Boot,
    Ready {
        window: Window,
        platform: PlatformContext,
        gl_context: Rc<GlContext>,
        handler: T,
    },
}

#[derive(Debug)]
pub struct Conf {
    pub window_attributes: WindowAttributes,
    /// Specifies the root folder, against which all fs request will be resolved.
    ///
    /// This setting has no effect on WASM. In WASM paths are resolved against the
    /// page url.
    pub fs_root: PathBuf,
    /// Configures the tracing filter.
    ///
    /// The default allows all tracing events with
    /// debug level or higher. You can also configure the filter through an environment
    /// variable. For more info, see [EnvFilter].
    ///
    /// Do not use the filter to disable debug! and trace! events altogether.
    /// Use tracing macros for setting max level instead
    pub filter: EnvFilter,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            window_attributes: default_window_attributes(),
            fs_root: PathBuf::new(),
            filter: default_log_filter(),
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
        .with_inner_size(LogicalSize::new(800, 600))
        .with_resizable(true)
        .with_title("Miniquad window")
        .with_window_icon(Some(default_icon))
}

pub fn default_log_filter() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(tracing::Level::DEBUG.into())
        .from_env()
        .expect("failed to parse log filter")
}

/// A trait defining event callbacks.
pub trait EventHandler: 'static {
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
