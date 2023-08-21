use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use bytes::{Bytes, BytesMut};
use futures::Future;
use futures_cancel::FutureExt;
use tokio::sync::Mutex;
use tower::{BoxError, Service};
use tower_rpc::Request;

use crate::history;

pub(crate) struct RequestHandler {
    rx: Arc<Mutex<history::Receiver>>,
}

impl RequestHandler {
    pub(crate) fn new(rx: history::Receiver) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

impl Service<Request<BytesMut>> for RequestHandler {
    type Response = Bytes;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<BytesMut>) -> Self::Future {
        let rx = self.rx.clone();
        let cancellation_token = req.context.cancellation_token();
        Box::pin(async move {
            Ok(
                match rx
                    .lock()
                    .await
                    .recv()
                    .cancel_on_shutdown(&cancellation_token)
                    .await
                {
                    Ok(Ok(log)) => Bytes::from(log),
                    _ => Bytes::default(),
                },
            )
        })
    }
}
