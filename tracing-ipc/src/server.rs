use std::{pin::Pin, task::Poll};

use bytes::{Bytes, BytesMut};
use futures::Future;
use futures_cancel::FutureExt;
use tokio::sync::broadcast;
use tower::{BoxError, Service};
use tower_rpc::Request;

#[derive(Clone)]
pub(crate) struct RequestHandler {
    tx: broadcast::Sender<Vec<u8>>,
}

impl RequestHandler {
    pub(crate) fn new(tx: broadcast::Sender<Vec<u8>>) -> Self {
        Self { tx }
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
        let mut rx = self.tx.subscribe();
        let cancellation_token = req.context.cancellation_token();
        Box::pin(async move {
            Ok(
                match rx.recv().cancel_on_shutdown(&cancellation_token).await {
                    Ok(Ok(log)) => Bytes::from(log),
                    _ => Bytes::default(),
                },
            )
        })
    }
}
