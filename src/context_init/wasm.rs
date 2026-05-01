use crate::context_init::NewSize;
use crate::{AppEvent, Conf};

use wasm_bindgen::JsValue;
use wasm_bindgen::convert::TryFromJsValue;
use wasm_bindgen::prelude::{Closure, ScopedClosure};
use web_sys::js_sys::{Array, Function};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlCanvasElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys};
use winit::window::Window;

// WebGL does not have a default precision mode. We must specify it explicitly.
pub const GLSL_VERSION: &str = "#version 300 es\nprecision mediump float;";

pub fn start_app<T>(event_loop: EventLoop<T>, app: impl ApplicationHandler<T> + 'static) {
    event_loop.spawn_app(app);
}

pub fn create_gfx_ctx_and_window(
    event_loop: &ActiveEventLoop,
    proxy: EventLoopProxy<AppEvent>,
    conf: &Conf,
) -> (Window, PlatformGfxContext) {
    let webgl_canvas = get_canvas();
    let window_attributes = conf.window_attributes.clone();
    let window = event_loop
        .create_window(window_attributes.with_canvas(Some(webgl_canvas.clone())))
        .expect("failed to create window");

    make_window_occupy_page(&window);
    spawn_size_observer(proxy);
    let webgl_context = get_canvas_webgl2_context(&webgl_canvas);

    (window, PlatformGfxContext { webgl_canvas, webgl_context })
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

fn make_window_occupy_page(gl_window: &Window) {
    let window = web_sys::window().expect("\"window\" not found");
    let document = window.document().expect("window has no \"document\"");
    let document = document
        .document_element()
        .expect("document has not document element");
    let size = get_parent_element_size(&document);
    let _ = gl_window.request_inner_size(size);
}

fn spawn_size_observer(proxy: EventLoopProxy<AppEvent>) {
    let window = web_sys::window().expect("\"window\" not found");
    let document = window.document().expect("window has no \"document\"");
    let document = document
        .document_element()
        .expect("document has not document element");
    let callback_closure: ScopedClosure<'static, dyn FnMut(JsValue, JsValue)> =
        Closure::new(move |entries: JsValue, _observer: JsValue| {
            let arr = Array::from(&entries);
            let entry = arr.get(0);
            let entry =
                ResizeObserverEntry::try_from_js_value(entry).expect("not an observer entry");
            let parent = entry.target();
            let sz = get_parent_element_size(&parent);
            let _ = proxy.send_event(AppEvent::NewSize(NewSize(sz)));
        });
    let callback = Function::from_closure(callback_closure);
    let observer = ResizeObserver::new(&callback).expect("failed to create the size observer");
    observer.observe(&document);

    std::mem::forget(callback);
}

fn get_parent_element_size(parent: &Element) -> PhysicalSize<u32> {
    let window = web_sys::window().expect("\"window\" not found");
    let dpi = window.device_pixel_ratio();
    let (width, height) = (
        (parent.client_width() as f64 * dpi).round(),
        (parent.client_height() as f64 * dpi).round(),
    );
    PhysicalSize::new(width as u32, height as u32)
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

pub struct PlatformGfxContext {
    webgl_canvas: HtmlCanvasElement,
    webgl_context: WebGl2RenderingContext,
}

impl PlatformGfxContext {
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
