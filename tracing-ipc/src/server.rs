use std::{path::Path, time::Duration};

use futures::StreamExt;
use parity_tokio_ipc::{Endpoint, SecurityAttributes};
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

use crate::command::Command;

pub(crate) async fn run_ipc_server(app_name: String, tx: tokio::sync::broadcast::Sender<Command>) {
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
    let mut buf = [0; 256];
    while let Some(result) = incoming.next().await {
        match result {
            Ok(stream) => {
                let (mut reader, mut writer) = split(stream);
                let mut rx = tx.subscribe();

                tokio::spawn(async move {
                    loop {
                        if (reader.read(&mut buf).await).is_err() {
                            return;
                        }
                        match rx.recv().await {
                            Ok(Command::Write(log)) => {
                                if (writer.write_all(&log).await).is_err() {
                                    return;
                                }
                            }
                            Ok(Command::Flush) => {
                                writer.flush().await.ok();
                                return;
                            }
                            Err(_) => return,
                        }
                    }
                });
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
