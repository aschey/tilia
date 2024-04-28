use std::error::Error;
use std::io;
use std::time::Duration;

use background_service::error::BoxedError;
use bytes::BytesMut;
use futures::{Future, Stream, StreamExt};

pub async fn run_client<F, S, E, Fut>(make_transport: F, tx: tokio::sync::mpsc::Sender<String>)
where
    F: Fn() -> Fut + Clone + Send + Sync,
    Fut: Future<Output = Result<S, BoxedError>> + Send,
    E: Error + Send + Sync,
    S: Stream<Item = Result<BytesMut, E>> + Send + Unpin + 'static,
{
    let make_client = || async {
        loop {
            if let Ok(client) = make_transport().await {
                break client;
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let mut client = make_client().await;
    loop {
        let send = async {
            let log_bytes = client
                .next()
                .await
                .ok_or(io::Error::new(io::ErrorKind::NotConnected, "end of stream"))??;
            if let Ok(log_str) = String::from_utf8(log_bytes.to_vec()) {
                let _ = tx.send(log_str).await;
            }
            Ok::<_, BoxedError>(())
        };

        if send.await.is_err() {
            client = make_client().await;
        }
    }
}
