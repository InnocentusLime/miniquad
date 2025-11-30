use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{JoinHandle, spawn};

use winit::event_loop::EventLoopProxy;

pub struct FsServerHandle(Sender<FsTask>);

impl FsServerHandle {
    pub fn submit_task(&self, path: &str, user_id: u64) {
        let path = path.to_string();
        self.0
            .send(FsTask { path, user_id })
            .expect("Worker thread terminated");
    }
}

// TODO: this works only for native
pub(crate) struct FsServer {
    _worker_thread: JoinHandle<()>,
    task_queue: Sender<FsTask>,
}

impl FsServer {
    pub(crate) fn start(event_loop_proxy: EventLoopProxy<FileReady>) -> FsServer {
        let (snd, rcv) = channel();
        let worker_thread = spawn(move || {
            fs_server_worker(rcv, event_loop_proxy);
        });
        FsServer {
            _worker_thread: worker_thread,
            task_queue: snd,
        }
    }

    pub fn get_handle(&self) -> FsServerHandle {
        FsServerHandle(self.task_queue.clone())
    }
}

fn fs_server_worker(task_queue: Receiver<FsTask>, proxy: EventLoopProxy<FileReady>) {
    while let Ok(task) = task_queue.recv() {
        let file_content: anyhow::Result<Vec<u8>> = std::fs::read(task.path).map_err(Into::into);
        proxy
            .send_event(FileReady {
                user_id: task.user_id,
                bytes_result: file_content,
            })
            .expect("Client terminated")
    }
}

struct FsTask {
    path: String,
    user_id: u64,
}

#[derive(Debug)]
pub struct FileReady {
    pub user_id: u64,
    pub bytes_result: anyhow::Result<Vec<u8>>,
}
