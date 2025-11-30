use std::{sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, TryRecvError}, thread::{spawn, JoinHandle}};

pub struct FsServerHandle(Sender<FsTask>);

impl FsServerHandle {
    pub fn submit_task<T: Send + 'static>(
        &self,
        path: &str,
        handler: impl FnOnce(Vec<u8>) -> anyhow::Result<T> + Send + 'static,
    ) -> FsTaskHandle<T> {
        let (snd, rcv) = sync_channel(1);
        self.0.send(FsTask{
            path: path.to_string(),
            response: Box::new(move |data| {
                fs_task_response(data, snd, handler);
            })
        }).expect("Worker thread terminated");
        FsTaskHandle(rcv)
    }
} 

// TODO: this works only for native
pub(crate) struct FsServer {
    _worker_thread: JoinHandle<()>,
    task_queue: Sender<FsTask>,
}

impl FsServer {
    pub(crate) fn start() -> FsServer {
        let (snd, rcv) = channel();
        let worker_thread = spawn(move || {
            fs_server_worker(rcv);
        });
        FsServer { _worker_thread: worker_thread, task_queue: snd }
    }

    pub fn get_handle(&self) -> FsServerHandle {
        FsServerHandle(self.task_queue.clone())
    }
}

fn fs_server_worker(task_queue: Receiver<FsTask>) {
    while let Ok(task) = task_queue.recv() {
        let file_content: anyhow::Result<Vec<u8>> = std::fs::read(task.path)
            .map_err(Into::into);
        (task.response)(file_content);
    }
}

fn fs_task_response<T>(
    data: anyhow::Result<Vec<u8>>,
    snd: SyncSender<anyhow::Result<T>>,
    handler: impl FnOnce(Vec<u8>) -> anyhow::Result<T> + Send,
) {
    let result = match data {
        Ok(x) => handler(x),
        Err(e) => Err(e),
    };
    // TODO: log that the user dropped their handle
    let _ = snd.send(result);
}

struct FsTask {
    path: String,
    response: Box<dyn FnOnce(anyhow::Result<Vec<u8>>) + Send>,
}

pub struct FsTaskHandle<T>(Receiver<anyhow::Result<T>>);

impl<T: Send + 'static> FsTaskHandle<T> {
    pub fn is_done(&self) -> Option<anyhow::Result<T>> {
        match self.0.try_recv() {
            Ok(x) => Some(x),
            Err(TryRecvError::Empty) => None,
            // FS worker terminating is an unrecoverable scenario
            Err(TryRecvError::Disconnected) => panic!("FS worker terminated"),
        }
    }
}
