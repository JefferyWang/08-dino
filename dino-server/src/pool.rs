use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use anyhow::Result;
use tracing::info;

use crate::{JsWorker, Req, Res};

pub struct Params {
    code: String,
    handler: String,
    req: Req,
    tx: oneshot::Sender<Result<Res>>,
}

#[allow(dead_code)]
pub struct WorkerPool {
    workers: Vec<JoinHandle<()>>,
    pub sender: Sender<Params>,
}

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Params>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for i in 1..=size {
            info!("create worker {}", i);
            let receiver = Arc::clone(&receiver);
            let handler = thread::spawn(move || loop {
                let Ok(guard) = receiver.lock() else {
                    break;
                };
                if let Ok(params) = guard.recv() {
                    info!("worker {} got a job", i);
                    let worker = JsWorker::try_new(&params.code).unwrap();
                    let res = worker.run(params.handler.as_str(), params.req);
                    params.tx.send(res).unwrap();
                }
            });
            workers.push(handler);
        }
        Self { workers, sender }
    }
}

impl Params {
    pub fn new(code: String, handler: String, req: Req, tx: oneshot::Sender<Result<Res>>) -> Self {
        Self {
            code,
            handler,
            req,
            tx,
        }
    }
}
