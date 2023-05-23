use std::time::Duration;

use bytes::{Bytes, BytesMut};
use tower::{reconnect::Reconnect, service_fn, util::BoxService, BoxError, ServiceExt};
use tower_rpc::{
    length_delimited_codec,
    transport::{ipc, tcp},
    Client, ClientError, IntoBoxedConnection, ReadyServiceExt,
};

use crate::TransportType;

pub async fn run_client(transport_type: &TransportType, tx: tokio::sync::mpsc::Sender<String>) {
    let make_client = service_fn(move |_: ()| {
        Box::pin(async move {
            let client_transport = match transport_type {
                TransportType::Ipc(app_name) => ipc::connect(app_name).await?.into_boxed(),
                TransportType::Tcp(socket_addr) => {
                    tcp::TcpStream::connect(socket_addr).await?.into_boxed()
                }
            };

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
