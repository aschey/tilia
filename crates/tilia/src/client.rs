use std::time::Duration;

use bytes::{Bytes, BytesMut};
use tower::{reconnect::Reconnect, service_fn, util::BoxService, BoxError, ServiceExt};
use tower_rpc::{length_delimited_codec, transport::ipc, Client, ClientError, ReadyServiceExt};

pub async fn run_ipc_client(app_name: &str, tx: tokio::sync::mpsc::Sender<String>) {
    let make_client = service_fn(move |_: ()| {
        Box::pin(async move {
            let client_transport = ipc::connect(app_name).await?;
            let client = Client::new(length_delimited_codec(client_transport)).create_pipeline();
            Ok::<_, BoxError>(client.boxed())
        })
    });
    let mut client =
        Reconnect::new::<BoxService<BytesMut, Bytes, ClientError>, ()>(make_client, ());

    loop {
        if let Ok(log_bytes) = client.call_ready(Bytes::default()).await {
            if let Ok(log_str) = String::from_utf8(log_bytes.to_vec()) {
                tx.send(log_str).await.ok();
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
