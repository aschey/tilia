use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::{Future, Sink, TryStream};
use tower::reconnect::Reconnect;
use tower::util::BoxService;
use tower::{service_fn, BoxError, Service, ServiceExt};
use tower_rpc::Client;

pub async fn run_client<F, S, Fut>(make_transport: F, tx: tokio::sync::mpsc::Sender<String>)
where
    F: Fn() -> Fut + Clone + Send + Sync,
    Fut: Future<Output = Result<S, BoxError>> + Send,
    S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <S as futures::TryStream>::Error: std::fmt::Debug,
    <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
{
    let make_client = service_fn(move |_: ()| {
        let make_transport = make_transport.clone();
        Box::pin(async move {
            let transport = make_transport().await?;
            let client = Client::new(transport).create_pipeline();
            Ok::<_, BoxError>(client.boxed())
        })
    });
    let mut client = Reconnect::new::<BoxService<BytesMut, Bytes, BoxError>, ()>(make_client, ());

    loop {
        let send = async {
            let log_bytes = client.ready().await?.call(Bytes::default()).await?;
            if let Ok(log_str) = String::from_utf8(log_bytes.to_vec()) {
                tx.send(log_str).await.ok();
            }
            Ok::<_, BoxError>(())
        };

        if send.await.is_err() {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
