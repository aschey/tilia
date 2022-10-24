use std::time::Duration;

use parity_tokio_ipc::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_ipc_client(app_name: &str, tx: tokio::sync::mpsc::Sender<String>) {
    #[cfg(unix)]
    let sock_path = format!("/tmp/{}_logs.sock", app_name);
    #[cfg(windows)]
    let sock_path = format!("\\\\.\\pipe\\{}_logs", app_name);
    let mut buf = [0; 1024 * 8];
    loop {
        match Endpoint::connect(&sock_path).await {
            Ok(mut client) => loop {
                if client.write_all(b" ").await.is_err() {
                    break;
                }
                let bytes_read = match client.read(&mut buf).await {
                    Ok(bytes_read) => bytes_read,
                    Err(_) => break,
                };
                let buf_str = std::str::from_utf8(&buf[0..bytes_read]).unwrap();
                let logs: Vec<_> = buf_str.split('\n').collect();
                for log in logs {
                    if !log.is_empty() && tx.send(log.to_owned()).await.is_err() {
                        return;
                    }
                }
            },
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
