use background_service::error::BoxedError;
use background_service::{BackgroundService, ServiceContext};
use bytes::Bytes;
use futures::{Sink, SinkExt, Stream, StreamExt};
use futures_cancel::FutureExt;

use crate::history;

pub(crate) struct RequestHandler<S, I, E>
where
    S: Stream<Item = Result<I, E>>,
{
    tx: history::Sender,
    transport: S,
}

impl<S, I, E> RequestHandler<S, I, E>
where
    S: Stream<Item = Result<I, E>>,
{
    pub(crate) fn new(transport: S, tx: history::Sender) -> Self {
        Self { tx, transport }
    }
}

impl<S, I, E> BackgroundService for RequestHandler<S, I, E>
where
    S: Stream<Item = Result<I, E>> + Send,
    I: Sink<Bytes> + Unpin + Send + 'static,
{
    fn name(&self) -> &str {
        "request_handler"
    }

    async fn run(self, mut context: ServiceContext) -> Result<(), BoxedError> {
        let transport = self.transport;
        futures::pin_mut!(transport);
        let token = context.cancellation_token();
        while let Ok(Some(Ok(mut client))) = transport.next().cancel_on_shutdown(&token).await {
            let mut rx = self.tx.subscribe();
            context.add_service(("request", |context: ServiceContext| async move {
                let token = context.cancellation_token();
                while let Ok(Ok(msg)) = rx.recv().cancel_on_shutdown(&token).await {
                    let _ = client.send(Bytes::from(msg)).await;
                }
                Ok(())
            }));
        }
        Ok(())
    }
}
