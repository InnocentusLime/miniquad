use crate::AppEvent;

use super::{FileReady, FsServer};

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::js_sys::{JSON, Uint8Array};
use web_sys::{Response, Window};
use winit::event_loop::EventLoopProxy;

pub(crate) fn spawn_fs_server(
    event_loop_proxy: EventLoopProxy<AppEvent>,
    fs_root: PathBuf,
) -> Rc<dyn FsServer> {
    let window = web_sys::window().expect("window inaccessible");
    let page_url_pathname = window.location().pathname().expect("no url pathname");
    let mut page_url_pathname = Path::new(&page_url_pathname).to_path_buf();
    if page_url_pathname.extension() == Some(OsStr::new("html")) {
        page_url_pathname.pop();
    }
    Rc::new(WasmFsServer { window, fs_root, page_url_pathname, event_loop_proxy })
}

struct WasmFsServer {
    window: Window,
    page_url_pathname: PathBuf,
    fs_root: PathBuf,
    event_loop_proxy: EventLoopProxy<AppEvent>,
}

impl FsServer for WasmFsServer {
    fn load_file(&self, path: &Path) {
        let path: Rc<Path> = path.into();
        let url = PathBuf::from_iter([&self.page_url_pathname, &self.fs_root, &*path])
            .to_string_lossy()
            .into_owned();

        let resolve_path = path.clone();
        let resolve_proxy = self.event_loop_proxy.clone();
        let resolve_callback = Closure::once(move |val| {
            match fetch_handler_impl(val, resolve_path.clone(), resolve_proxy.clone()) {
                Ok(()) => Ok(()),
                Err(e) => send_result(Err(e), resolve_path, &resolve_proxy),
            }
        });

        let reject_path = path.clone();
        let reject_proxy = self.event_loop_proxy.clone();
        let reject_callback = Closure::once(move |val| {
            // fetch fails only when we couldn't establish a connection or CORS kicks in.
            let err = io::Error::new(io::ErrorKind::NetworkUnreachable, js_val_to_errmsg(val));
            send_result(Err(err), reject_path, &reject_proxy)
        });

        let _ = self
            .window
            .fetch_with_str(&url)
            .then_with_reject(&resolve_callback, &reject_callback);
        // Handover the closures to JS. They will be GC'd once the request finishes.
        resolve_callback.forget();
        reject_callback.forget();
    }
}

fn fetch_handler_impl(
    val: JsValue,
    path: Rc<Path>,
    proxy: EventLoopProxy<AppEvent>,
) -> io::Result<()> {
    let response = val
        .dyn_into::<web_sys::Response>()
        .map_err(|err| js_val_to_io_error("failed to cast fetch() result", err))?;

    check_http_status(&response)?;
    let array_buffer = response
        .array_buffer()
        .map_err(|err| js_val_to_io_error("failed to get response body", err))?;

    let resolve_path = path.clone();
    let resolve_proxy = proxy.clone();
    let resolve_callback = Closure::once(move |val| {
        let bytes = Uint8Array::new(&val).to_vec();
        send_result(Ok(bytes), resolve_path, &resolve_proxy)
    });

    let reject_path = path.clone();
    let reject_proxy = proxy.clone();
    let reject_callback = Closure::once(move |val| {
        // array_buffer fails only when the connection cuts off.
        let err = io::Error::new(io::ErrorKind::ConnectionAborted, js_val_to_errmsg(val));
        send_result(Err(err), reject_path, &reject_proxy)
    });

    let _ = array_buffer.then_with_reject(&resolve_callback, &reject_callback);
    // Handover the closures to JS. They will be GC'd once the request finishes.
    resolve_callback.forget();
    reject_callback.forget();
    Ok(())
}

fn check_http_status(response: &Response) -> io::Result<()> {
    let mk_err_simpl = |kind| Err(io::Error::new(kind, response.status_text()));
    match response.status() {
        // Mimiq expects assets to be hosted on a simple static server.
        // Only the 200 status makes sense for us.
        // Everything else singals a complicated API we aren't designed for.
        200 => Ok(()),
        404 | 410 => mk_err_simpl(io::ErrorKind::NotFound),
        400 | 405 | 406 | 411..=417 | 421..=426 | 428 | 431 => {
            mk_err_simpl(io::ErrorKind::InvalidInput)
        }
        401 | 402 | 403 | 407 | 451 | 511 => mk_err_simpl(io::ErrorKind::PermissionDenied),
        500..=510 | 512..=599 => mk_err_simpl(io::ErrorKind::NetworkDown),
        unexpected => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "unexpected request status: {unexpected} ({})",
                response.status_text()
            ),
        )),
    }
}

fn send_result(
    bytes_result: io::Result<Vec<u8>>,
    path: Rc<Path>,
    proxy: &EventLoopProxy<AppEvent>,
) -> Result<(), JsError> {
    // Let event loop closing bubble up as an unhandled promise error.
    // This way the error cleanly shows up in the console without creating a Rust panic.
    proxy
        .send_event(AppEvent::FileReady(FileReady {
            path: path.to_path_buf(),
            bytes_result,
        }))
        .map_err(JsError::from)
}

fn js_val_to_io_error(preamble: &str, err: JsValue) -> io::Error {
    let details = js_val_to_errmsg(err);
    io::Error::new(io::ErrorKind::Other, format!("{preamble}: {details}"))
}

fn js_val_to_errmsg(val: JsValue) -> String {
    match JSON::stringify(&val) {
        Ok(js_str) => format!("{js_str}"),
        Err(_) => "no details".to_string(),
    }
}
