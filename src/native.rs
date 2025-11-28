use std::{num::NonZeroU32, rc::Rc};

use crate::{ conf::Conf, EventHandler, GlContext};

use glutin::config::{Api, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext} ;
use glutin::context::{NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler; 
use winit::dpi::LogicalSize; 
use winit::event::WindowEvent; 
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop}; 
use winit::window::{Window, WindowAttributes};

struct App<F> {
    conf: Conf,
    window: Option<Window>,
    maker: Option<F>,
    surface: Option<Surface<WindowSurface>>,
    handler: Option<Box<dyn EventHandler>>,
    gl_context: Option<PossiblyCurrentContext>,
    internal_ctx: Option<Rc<GlContext>>,
}

impl<F> ApplicationHandler for App<F> 
where 
    F: 'static + FnOnce(Rc<GlContext>) -> Box<dyn EventHandler>, 
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = window_attributes(&self.conf);
        let display_builder = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes));
        let template_builder = ConfigTemplateBuilder::new()
            .with_api(Api::OPENGL)
            .with_alpha_size(8);
        let (window, gl_config) = display_builder.build(event_loop, template_builder, |mut conf| {
            conf.next().unwrap()
        }).unwrap();
        let window = window.unwrap();

        let gl_display = gl_config.display();
        let raw_window_handle = window.window_handle().unwrap().as_raw();
        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let gl_context = unsafe { gl_display.create_context(
            &gl_config, 
            &context_attributes,
        ).unwrap()
        .treat_as_possibly_current() };

        let attrs = window.build_surface_attributes(Default::default()).unwrap();
        let gl_surface = unsafe { gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap() };
        gl_context.make_current(&gl_surface).unwrap();
        let glow_gl = unsafe { glow::Context::from_loader_function(|proc| {
            let cstr = std::ffi::CString::new(proc).unwrap();
            gl_display.get_proc_address(&cstr)
        }) };
        
        let interval = match self.conf.platform.swap_interval {
            Some(x) => SwapInterval::Wait(NonZeroU32::new(x as u32).unwrap()),
            None => SwapInterval::DontWait,
        };
        gl_surface
            .set_swap_interval(&gl_context, interval)
            .unwrap();
        let ctx = Rc::new(GlContext::new(
            glow_gl,
            window.inner_size().into(),
        ));

        self.window = Some(window);
        self.handler = Some((self.maker.take().unwrap())(ctx.clone()));
        self.surface = Some(gl_surface);
        self.gl_context = Some(gl_context);
        self.internal_ctx = Some(ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                let Some(handler) = self.handler.as_mut() else {
                    return;
                };
                handler.quit_requested_event();
            },
            WindowEvent::RedrawRequested => {
                let Some(handler) = self.handler.as_mut() else {
                    return;
                };
                handler.draw();
                self.surface.as_ref().unwrap().swap_buffers(self.gl_context.as_ref().unwrap())
                    .unwrap();
            },
            WindowEvent::Resized(new_size) => {
                let ctx = self.internal_ctx.as_ref().unwrap();
                ctx.client_area_size.set(new_size.into());
                self.surface.as_ref().unwrap()
                    .resize(
                        self.gl_context.as_ref().unwrap(),
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}

pub fn run<F>(conf: Conf, f: F)
where
    F: 'static + FnOnce(Rc<GlContext>) -> Box<dyn EventHandler>,
{
    let event_loop = EventLoop::builder()
        .build()
        .unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    return event_loop.run_app(&mut App {
        conf,
        window: None,
        maker: Some(f),
        handler: None,
        surface: None,
        gl_context: None,
        internal_ctx: None,
    }).unwrap();
}

fn window_attributes(conf: &Conf) -> WindowAttributes {
    let size = LogicalSize::new(conf.window_width, conf.window_height);
    Window::default_attributes()
        .with_inner_size(size)
        .with_title(&conf.window_title)
        .with_resizable(conf.window_resizable)
}
