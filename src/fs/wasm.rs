use std::{
    io,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use crate::AppEvent;

use super::{FileReady, FsServer, TARGET_NAME};

use wasm_bindgen::prelude::*;
use web_sys::Response;
use web_sys::js_sys::{JSON, Uint8Array};
use winit::event_loop::EventLoopProxy;

pub(crate) fn spawn_fs_server(
    event_loop_proxy: EventLoopProxy<AppEvent>,
    fs_root: PathBuf,
) -> Rc<dyn FsServer> {
    let window = web_sys::window().expect("\"window\" not found");
    let page_url = window
        .location()
        .pathname()
        .expect("Could not get location.pathname");
    let mut page_url = PathBuf::from_str(&page_url).expect("Page url parse");
    if let Some(ext) = page_url.extension()
        && ext == "html"
    {
        page_url.pop();
    }

    Rc::new(WasmFsServer { fs_root, page_url, event_loop_proxy })
}

struct WasmFsServer {
    page_url: PathBuf,
    fs_root: PathBuf,
    event_loop_proxy: EventLoopProxy<AppEvent>,
}

impl FsServer for WasmFsServer {
    fn load_file(&self, path: &Path) {
        let path: Rc<Path> = path.into();
        tracing::debug!(target: TARGET_NAME, path=?path, "will load");

        let url = PathBuf::from_iter([&self.page_url, &self.fs_root, &*path])
            .to_string_lossy()
            .into_owned();
        let window = web_sys::window().expect("\"window\" not found");
        let proxy = self.event_loop_proxy.clone();
        let then_callback = Closure::new(move |val| fetch_handler(val, path.clone(), &proxy));

        let _ = window.fetch_with_str(&url).then(&then_callback);
        then_callback.forget();
    }
}

fn fetch_handler(val: JsValue, path: Rc<Path>, proxy: &EventLoopProxy<AppEvent>) {
    tracing::debug!(target: TARGET_NAME, path=?path, "response received");
    if let Err(e) = fetch_handler_impl(val, path.clone(), proxy.clone()) {
        tracing::error!(target: TARGET_NAME, "{e:#}");
        proxy
            .send_event(AppEvent::FileReady(FileReady {
                path: path.to_path_buf(),
                bytes_result: Err(e),
            }))
            .expect("Loop died");
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
    let then_closure = Closure::new(move |val| array_buffer_handler(val, path.clone(), &proxy));
    let _ = array_buffer.then(&then_closure);
    then_closure.forget();
    Ok(())
}

fn array_buffer_handler(val: JsValue, path: Rc<Path>, proxy: &EventLoopProxy<AppEvent>) {
    tracing::debug!(target: TARGET_NAME, path=?path, "done");
    let bytes = Uint8Array::new(&val).to_vec();
    proxy
        .send_event(AppEvent::FileReady(FileReady {
            path: path.to_path_buf(),
            bytes_result: Ok(bytes),
        }))
        .expect("Loop died");
}

fn check_http_status(response: &Response) -> io::Result<()> {
    match response.status() {
        200 => Ok(()),
        404 | 410 => Err(io::Error::new(
            io::ErrorKind::NotFound,
            response.status_text(),
        )),
        400 | 405 | 406 | 411 | 412 | 413 | 414 | 415 | 416 | 417 | 421 | 422 | 423 | 424 | 425
        | 426 | 428 | 431 => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            response.status_text(),
        )),
        401 | 402 | 403 | 407 | 451 | 511 => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            response.status_text(),
        )),
        500..511 | 512..600 => Err(io::Error::new(
            io::ErrorKind::NetworkDown,
            response.status_text(),
        )),
        unexpected => Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "unexpected request status: {unexpected} ({})",
                response.status_text()
            ),
        )),
    }
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
