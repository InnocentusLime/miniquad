use std::{
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use crate::AppEvent;

use super::{FileReady, TARGET_NAME};

use wasm_bindgen::prelude::*;
use web_sys::js_sys::{JSON, Uint8Array};
use winit::event_loop::EventLoopProxy;

pub struct FsServerHandle {
    page_url: PathBuf,
    fs_root: PathBuf,
    event_loop_proxy: EventLoopProxy<AppEvent>,
}

impl FsServerHandle {
    pub fn load_file(&self, path: impl AsRef<Path>) {
        let path: Rc<Path> = path.as_ref().into();
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

pub(crate) struct FsServer {
    page_url: PathBuf,
    fs_root: PathBuf,
    event_loop_proxy: EventLoopProxy<AppEvent>,
}

impl FsServer {
    pub(crate) fn start(event_loop_proxy: EventLoopProxy<AppEvent>, fs_root: PathBuf) -> FsServer {
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
        FsServer { fs_root, page_url, event_loop_proxy }
    }

    pub fn get_handle(&self) -> FsServerHandle {
        FsServerHandle {
            page_url: self.page_url.clone(),
            fs_root: self.fs_root.clone(),
            event_loop_proxy: self.event_loop_proxy.clone(),
        }
    }
}

fn fetch_handler(val: JsValue, path: Rc<Path>, proxy: &EventLoopProxy<AppEvent>) {
    tracing::debug!(target: TARGET_NAME, path=?path, "response received");
    if let Err(e) = fetch_handler_impl(val, path.clone(), proxy.clone()) {
        tracing::error!(target: TARGET_NAME, "{e:?}");
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
) -> anyhow::Result<()> {
    let response = match val.dyn_into::<web_sys::Response>() {
        Ok(x) => x,
        Err(e) => anyhow::bail!("failed to get a response: {}", js_val_to_errmsg(e)),
    };
    let status = response.status();
    if status != 200 {
        anyhow::bail!("failed request with status {status}");
    }
    let array_buffer = match response.array_buffer() {
        Ok(x) => x,
        Err(e) => anyhow::bail!(
            "failed to get the response body bytes: {}",
            js_val_to_errmsg(e)
        ),
    };
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

fn js_val_to_errmsg(val: JsValue) -> String {
    match JSON::stringify(&val) {
        Ok(js_str) => format!("{js_str}"),
        Err(_) => "no details".to_string(),
    }
}
