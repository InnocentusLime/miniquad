use crate::AppEvent;

use super::{FileReady, FsServer};

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{JoinHandle, spawn};

use winit::event_loop::EventLoopProxy;

pub(crate) fn spawn_fs_server(
    event_loop_proxy: EventLoopProxy<AppEvent>,
    fs_root: PathBuf,
) -> Rc<dyn FsServer> {
    let (snd, rcv) = channel();
    let worker_thread = spawn(move || {
        fs_server_worker(rcv, event_loop_proxy);
    });
    Rc::new(NativeFsServer { _worker_thread: worker_thread, task_queue: snd, fs_root })
}

struct NativeFsServer {
    _worker_thread: JoinHandle<()>,
    task_queue: Sender<FsTask>,
    fs_root: PathBuf,
}

impl FsServer for NativeFsServer {
    fn load_file(&self, path: &Path) {
        let orig_path = path.to_path_buf();
        let path = self.fs_root.join(path);
        self.task_queue
            .send(FsTask { path, orig_path })
            .expect("Worker thread terminated");
    }
}

fn fs_server_worker(task_queue: Receiver<FsTask>, proxy: EventLoopProxy<AppEvent>) {
    while let Ok(task) = task_queue.recv() {
        let file_content = std::fs::read(&task.path);
        let send_res = proxy.send_event(AppEvent::FileReady(FileReady {
            path: task.orig_path,
            bytes_result: file_content,
        }));
        // Err is EventLoopClosed. Shutdown properly.
        if send_res.is_err() {
            break;
        }
    }
}

#[derive(Debug)]
struct FsTask {
    orig_path: PathBuf,
    path: PathBuf,
}
