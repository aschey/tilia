use std::{path::Path, time::Duration};

use futures::StreamExt;
use parity_tokio_ipc::{Endpoint, SecurityAttributes};
use tokio::{
    io::{split, AsyncReadExt},
    sync::mpsc::Sender,
};

pub async fn run_ipc_server(app_name: &str, tx: Sender<String>) {
    #[cfg(unix)]
    let sock_name = format!("/tmp/{app_name}_logs.sock");

    #[cfg(windows)]
    let sock_name = format!("\\\\.\\pipe\\{app_name}_logs");

    // Need to make sure path doesn't exist before connecting
    let socket_path = Path::new(&sock_name);
    if socket_path.exists() {
        std::fs::remove_file(socket_path).ok();
    }

    let mut endpoint = Endpoint::new(sock_name);
    endpoint.set_security_attributes(SecurityAttributes::allow_everyone_create().unwrap());

    let incoming = endpoint.incoming().expect("failed to open new socket");
    futures::pin_mut!(incoming);
    let mut buf = [0; 1024 * 8];
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

                    let buf_str = std::str::from_utf8(&buf[0..bytes]).unwrap();
                    let logs: Vec<_> = buf_str.split('\n').collect();
                    for log in logs {
                        if !log.is_empty() && tx.send(log.to_owned()).await.is_err() {
                            return;
                        }
                    }
                }
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
