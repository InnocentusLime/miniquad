use crate::Conf;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys};
use winit::window::Window;

// WebGL does not have a default precision mode. We must specify it explicitly.
pub const GLSL_VERSION: &str = "#version 300 es\nprecision mediump float;";

pub fn start_app<T>(event_loop: EventLoop<T>, app: impl ApplicationHandler<T> + 'static) {
    event_loop.spawn_app(app);
}

pub fn create_ctx_and_window(
    event_loop: &ActiveEventLoop,
    conf: &Conf,
) -> (Window, PlatformContext) {
    let webgl_canvas = get_canvas();
    let window_attributes = conf.window_attributes.clone();
    let window = event_loop
        .create_window(window_attributes.with_canvas(Some(webgl_canvas.clone())))
        .expect("failed to create window");

    make_window_occupy_page(&window);
    let webgl_context = get_canvas_webgl2_context(&webgl_canvas);

    (
        window,
        PlatformContext {
            webgl_canvas,
            webgl_context,
        },
    )
}

fn get_canvas() -> HtmlCanvasElement {
    let window = web_sys::window().expect("\"window\" not found");
    let document = window.document().expect("window has no \"document\"");
    let canvas_elemnt = document
        .get_element_by_id("app_canvas")
        .expect("No element with app_canvas id");
    canvas_elemnt
        .dyn_into::<_>()
        .expect("app_canvas element is not a canvas")
}

// FIXME: hacky and doesn't respond to element size changes
fn make_window_occupy_page(gl_window: &Window) {
    let window = web_sys::window().expect("\"window\" not found");
    let document = window.document().expect("window has no \"document\"");
    let document = document
        .document_element()
        .expect("document has not document element");
    let dpi = window.device_pixel_ratio();
    let (width, height) = (
        (document.client_width() as f64 * dpi).round(),
        (document.client_height() as f64 * dpi).round(),
    );
    let _ = gl_window.request_inner_size(PhysicalSize::new(width as u32, height as u32));
}

fn get_canvas_webgl2_context(webgl_canvas: &HtmlCanvasElement) -> WebGl2RenderingContext {
    let context_object = webgl_canvas
        .get_context("webgl2")
        .expect("failed to create WebGL2 context")
        .expect("No WebGL context");
    context_object
        .dyn_into::<_>()
        .expect("received context is not a WebGL context")
}

pub struct PlatformContext {
    webgl_canvas: HtmlCanvasElement,
    webgl_context: WebGl2RenderingContext,
}

impl PlatformContext {
    pub fn resize_surface(&self, new_size: PhysicalSize<u32>) {
        self.webgl_canvas.set_width(new_size.width);
        self.webgl_canvas.set_height(new_size.height);
    }

    pub fn make_glow_context(&self, _is_debug: bool) -> glow::Context {
        glow::Context::from_webgl2_context(self.webgl_context.clone())
    }

    pub fn swap_buffers(&self) {
        /* NO-OP */
    }
}
