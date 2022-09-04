use std::time::Duration;

use futures::StreamExt;
use parity_tokio_ipc::{Endpoint, SecurityAttributes};
use tokio::{
    io::{split, AsyncReadExt},
    sync::mpsc::Sender,
};

pub async fn run_ipc_server(app_name: &str, tx: Sender<String>) {
    #[cfg(unix)]
    let sock_name = format!("/tmp/{app_name}.sock");
    #[cfg(windows)]
    let sock_name = format!("\\\\.\\pipe\\{app_name}");

    let mut endpoint = Endpoint::new(sock_name);
    endpoint.set_security_attributes(SecurityAttributes::allow_everyone_create().unwrap());

    let incoming = endpoint.incoming().expect("failed to open new socket");
    futures::pin_mut!(incoming);
    let mut buf = [0; 2048];
    while let Some(result) = incoming.next().await {
        match result {
            Ok(stream) => {
                let (mut reader, _) = split(stream);

                loop {
                    let bytes = match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(bytes) => bytes,
                        Err(_) => break,
                    };

                    if tx
                        .send(std::str::from_utf8(&buf[0..bytes]).unwrap().to_string())
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
