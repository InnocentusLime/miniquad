use crate::AppEvent;

use super::{FileReady, TARGET_NAME};

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{JoinHandle, spawn};

use winit::event_loop::EventLoopProxy;

pub struct FsServerHandle {
    task_queue: Sender<FsTask>,
    fs_root: PathBuf,
}

impl FsServerHandle {
    pub fn load_file(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        tracing::info!(target: TARGET_NAME, path=?path, "will load");

        let orig_path = path.to_path_buf();
        let path = self.fs_root.join(path);
        tracing::debug!(
            target: TARGET_NAME,
            real_path=?path,
            "sending task to read file"
        );
        self.task_queue
            .send(FsTask { path, orig_path })
            .expect("Worker thread terminated");
    }
}

pub(crate) struct FsServer {
    _worker_thread: JoinHandle<()>,
    task_queue: Sender<FsTask>,
    fs_root: PathBuf,
}

impl FsServer {
    pub(crate) fn start(event_loop_proxy: EventLoopProxy<AppEvent>, fs_root: PathBuf) -> FsServer {
        let (snd, rcv) = channel();
        let worker_thread = spawn(move || {
            fs_server_worker(rcv, event_loop_proxy);
        });
        FsServer { _worker_thread: worker_thread, task_queue: snd, fs_root }
    }

    pub fn get_handle(&self) -> FsServerHandle {
        FsServerHandle { task_queue: self.task_queue.clone(), fs_root: self.fs_root.clone() }
    }
}

fn fs_server_worker(task_queue: Receiver<FsTask>, proxy: EventLoopProxy<AppEvent>) {
    while let Ok(task) = task_queue.recv() {
        let file_content: anyhow::Result<Vec<u8>> = std::fs::read(&task.path).map_err(Into::into);
        match &file_content {
            Ok(_) => tracing::info!(target: TARGET_NAME, path=?task.path, "done"),
            Err(e) => tracing::error!(target: TARGET_NAME, path=?task.path, "{e:#}"),
        }

        let send_res = proxy.send_event(AppEvent::FileReady(FileReady {
            path: task.orig_path,
            bytes_result: file_content,
        }));
        if send_res.is_err() {
            tracing::debug!(target: TARGET_NAME, "terminating: event loop closed");
        }
    }
    tracing::debug!(target: TARGET_NAME, "terminated");
}

#[derive(Debug)]
struct FsTask {
    orig_path: PathBuf,
    path: PathBuf,
}
