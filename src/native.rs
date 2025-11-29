use std::{num::NonZeroU32, rc::Rc};

use crate::{ conf::Conf, EventHandler, GlContext};

use glutin::config::{Api, Config, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext} ;
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler; 
use winit::dpi::LogicalSize; 
use winit::event::WindowEvent; 
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop}; 
use winit::window::{Window, WindowAttributes};

pub fn run<Init, Handler>(conf: Conf, init: Init)
where
    Init: 'static + FnOnce(Rc<GlContext>) -> Handler,
    Handler: EventHandler,
{
    let event_loop = EventLoop::builder()
        .build()
        .unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    return event_loop.run_app(&mut App {
        conf,
        state: AppState::Boot { init },
    }).unwrap();
}

struct App<Init, Handler> {
    conf: Conf,
    state: AppState<Init, Handler>,
}

impl<Init, Handler> ApplicationHandler for App<Init, Handler> 
where 
    Init: 'static + FnOnce(Rc<GlContext>) -> Handler,
    Handler: EventHandler,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match &mut self.state {
            AppState::Boot { .. } => {
                let init = match std::mem::replace(&mut self.state, AppState::Initing) {
                    AppState::Boot { init } => init,
                    AppState::Initing => unreachable!("Expected AppState to be \"Boot\", got \"Initing\""),
                    AppState::Ready { .. } => unreachable!("Expected AppState to be \"Boot\", got \"Ready\""),
                };
                self.state = Self::prepare(event_loop, &self.conf, init);
            },
            AppState::Ready { .. } => unimplemented!("Restoring of applications is not supported"),
            AppState::Initing => panic!("Resumed while initing"),
        }
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
                AppState::Ready { surface, gl_context, .. }
            ) => {
                gl_context.client_area_size.set((*new_size).into());
                surface.resize(
                        &gl_context.glutin_ctx,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
            },
            _ => (),
        }
        
        let AppState::Ready { 
            window, 
            handler, 
            surface, 
            gl_context 
        } = &mut self.state else {
            return;
        };
        let swap_buffers = matches!(event, WindowEvent::RedrawRequested);
        handler.window_event(event, window);
        if swap_buffers {
            surface.swap_buffers(&gl_context.glutin_ctx)
                .expect("Failed to swap buffers");
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let AppState::Ready { window, handler, .. } = &mut self.state else {
            return;
        };
        window.request_redraw();
        handler.update();
    }
}

impl<Init, Handler> App<Init, Handler> 
where 
    Init: 'static + FnOnce(Rc<GlContext>) -> Handler,
    Handler: EventHandler,
{
    fn prepare(event_loop: &ActiveEventLoop, conf: &Conf, init: Init) -> AppState<Init, Handler> {
        let (window, display, gl_config) = create_window_and_gl_config(event_loop, conf);
        let (gl_context, surface) = create_surface_and_context(&display, &gl_config, &window, conf);
        
        gl_context.make_current(&surface).unwrap();
        let glow_gl = unsafe { 
            glow::Context::from_loader_function_cstr(|proc| display.get_proc_address(proc)) 
        };
        let gl_context = Rc::new(GlContext::new(
            gl_context,
            glow_gl,
            window.inner_size().into(),
        ));
        
        let handler = init(gl_context.clone());
        AppState::Ready { window, surface, gl_context, handler }
    }
}

enum AppState<Init, Handler> {
    Boot {
        init: Init,
    },
    Initing,
    Ready {
        window: Window,
        surface: Surface<WindowSurface>,
        gl_context: Rc<GlContext>,
        handler: Handler,
    },
}

fn create_window_and_gl_config(event_loop: &ActiveEventLoop, conf: &Conf) -> (Window, Display, Config) {
    let window_attributes = window_attributes(conf);
    let display_builder = DisplayBuilder::new()
        .with_window_attributes(Some(window_attributes));
    let template_builder = ConfigTemplateBuilder::new()
        .with_api(Api::OPENGL)
        .with_alpha_size(8);
    let (window, gl_config) = display_builder.build(event_loop, template_builder, |mut conf| {
        conf.next().expect("No GL configuration found")
    }).expect("Could not initialize the GL platform");
    let window = window.expect("No window has been created");
    
    (window, gl_config.display(), gl_config)
}

fn window_attributes(conf: &Conf) -> WindowAttributes {
    let size = LogicalSize::new(conf.window_width, conf.window_height);
    Window::default_attributes()
        .with_inner_size(size)
        .with_title(&conf.window_title)
        .with_resizable(conf.window_resizable)
}

fn create_surface_and_context(
    gl_display: &Display, 
    gl_config: &Config, 
    window: &Window,
    conf: &Conf,
) -> (PossiblyCurrentContext, Surface<WindowSurface>) {
    let raw_window_handle = window.window_handle().expect("Window has not raw handle").as_raw();
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
    let gl_context = unsafe { gl_display.create_context(
        &gl_config, 
        &context_attributes,
    ).expect("Failed to create GL context") };
    let gl_context = gl_context.treat_as_possibly_current();
    
    let surface_attributes = window.build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");
    let surface = unsafe { gl_display
        .create_window_surface(&gl_config, &surface_attributes)
        .expect("Failed to create window surface") };
    gl_context.make_current(&surface).expect("Failed to make the context current");
    surface
        .set_swap_interval(&gl_context, conf.swap_interval)
        .expect("Failed to update window swap interval");

    (gl_context, surface)
}
