use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;

pub fn channel(capacity: usize) -> Sender {
    let (tx, _) = broadcast::channel(capacity);
    Sender {
        size: 0,
        capacity,
        buf: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
        tx,
    }
}

#[derive(Clone)]
pub struct Sender {
    size: usize,
    capacity: usize,
    buf: Arc<RwLock<VecDeque<Vec<u8>>>>,
    tx: broadcast::Sender<Vec<u8>>,
}

impl Sender {
    pub fn send(&mut self, value: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
        let mut buf = self.buf.write().expect("Lock poisoned");
        if self.size >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(value.clone());
        let res = self.tx.send(value);
        if self.size < self.capacity {
            self.size += 1;
        }
        res
    }

    pub fn subscribe(&self) -> Receiver {
        Receiver {
            buf: self.buf.read().expect("Lock poisoned").clone(),
            rx: self.tx.subscribe(),
        }
    }
}

pub struct Receiver {
    buf: VecDeque<Vec<u8>>,
    rx: broadcast::Receiver<Vec<u8>>,
}

impl Receiver {
    pub async fn recv(&mut self) -> Result<Vec<u8>, broadcast::error::RecvError> {
        if let Some(val) = self.buf.pop_front() {
            Ok(val)
        } else {
            self.rx.recv().await
        }
    }
}
