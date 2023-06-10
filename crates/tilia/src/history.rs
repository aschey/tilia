use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn channel(capacity: usize) -> Sender {
    let (tx, _) = broadcast::channel(capacity);
    Sender {
        buf: Arc::new(ArrayQueue::new(capacity)),
        tx,
    }
}

#[derive(Clone)]
pub struct Sender {
    buf: Arc<ArrayQueue<Vec<u8>>>,
    tx: broadcast::Sender<Vec<u8>>,
}

impl Sender {
    pub fn send(&mut self, value: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
        self.buf.force_push(value.clone());
        self.tx.send(value)
    }

    pub fn subscribe(&self) -> Receiver {
        Receiver {
            buf: self.buf.clone(),
            rx: self.tx.subscribe(),
        }
    }
}

pub struct Receiver {
    buf: Arc<ArrayQueue<Vec<u8>>>,
    rx: broadcast::Receiver<Vec<u8>>,
}

impl Receiver {
    pub async fn recv(&mut self) -> Result<Vec<u8>, broadcast::error::RecvError> {
        if let Some(val) = self.buf.pop() {
            Ok(val)
        } else {
            self.rx.recv().await
        }
    }
}
