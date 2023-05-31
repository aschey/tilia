use std::sync::{Arc, RwLock};

use arraydeque::{ArrayDeque, Wrapping};
use tokio::sync::broadcast;

pub fn channel<const CAP: usize>() -> Sender<CAP> {
    let (tx, _) = broadcast::channel(CAP);
    Sender {
        buf: Default::default(),
        tx,
    }
}

#[derive(Clone)]
pub struct Sender<const CAP: usize> {
    buf: Arc<RwLock<ArrayDeque<Vec<u8>, CAP, Wrapping>>>,
    tx: broadcast::Sender<Vec<u8>>,
}

impl<const CAP: usize> Sender<CAP> {
    pub fn send(&mut self, value: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
        self.buf
            .write()
            .expect("Lock poisoned")
            .push_back(value.clone());
        self.tx.send(value)
    }

    pub fn subscribe(&self) -> Receiver<CAP> {
        Receiver {
            buf: self.buf.read().expect("Lock poisoned").clone(),
            rx: self.tx.subscribe(),
        }
    }
}

pub struct Receiver<const CAP: usize> {
    buf: ArrayDeque<Vec<u8>, CAP, Wrapping>,
    rx: broadcast::Receiver<Vec<u8>>,
}

impl<const CAP: usize> Receiver<CAP> {
    pub async fn recv(&mut self) -> Result<Vec<u8>, broadcast::error::RecvError> {
        if let Some(val) = self.buf.pop_front() {
            Ok(val)
        } else {
            self.rx.recv().await
        }
    }
}
