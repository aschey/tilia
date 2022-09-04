use std::{sync::atomic::Ordering, time::Duration};

use parity_tokio_ipc::Endpoint;
use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};
use tracing_subscriber::fmt::MakeWriter;

use crate::{command::Command, state, WorkerGuard};

pub struct Writer;

impl Writer {
    pub fn new(app_name: &str) -> (Self, WorkerGuard) {
        #[cfg(unix)]
        let sock_name = format!("/tmp/{app_name}.sock");
        #[cfg(windows)]
        let sock_name = format!("\\\\.\\pipe\\{app_name}");

        if !state::IS_INITIALIZED.swap(true, Ordering::SeqCst) {
            let (tx, rx) = tokio::sync::mpsc::channel(32);
            state::SENDER.get_or_init(|| tx);

            Self::init(sock_name, rx);
        }
        (Self, WorkerGuard)
    }

    fn init(sock_name: String, mut rx: Receiver<Command>) {
        let handle = tokio::spawn(async move {
            loop {
                state::IS_CONNECTED.swap(false, Ordering::SeqCst);
                let mut last_connect = tokio::time::Instant::now();

                let mut client = match Endpoint::connect(&sock_name).await {
                    Ok(client) => client,
                    Err(_) => loop {
                        tokio::select! {
                            client = Endpoint::connect(&sock_name), if tokio::time::Instant::now().duration_since(last_connect) > tokio::time::Duration::from_secs(1) => {
                                match client {
                                    Ok(client) => break client,
                                    Err(_) => {
                                        last_connect = tokio::time::Instant::now();
                                    }
                                }
                            },
                            cmd = rx.recv() => {
                                match cmd {
                                    Some(Command::Write(_)) => {},
                                    _ => {
                                        return;
                                    },

                                }
                            },
                            _ =  tokio::time::sleep(Duration::from_millis(1000)) => {},
                        }
                    },
                };

                state::IS_CONNECTED.swap(true, Ordering::SeqCst);
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        Command::Write(buf) => {
                            if client.write_all(&buf).await.is_err() {
                                break;
                            };
                        }
                        Command::Flush => {
                            let _ = client.flush().await;
                            return;
                        }
                    }
                }
            }
        });
        state::HANDLE.set(handle).unwrap();
    }
}

impl MakeWriter<'_> for Writer {
    type Writer = Writer;

    fn make_writer(&'_ self) -> Self::Writer {
        Self
    }
}

impl std::io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let b = buf.to_owned();

        tokio::spawn(async move {
            if let Err(e) = state::SENDER.get().unwrap().send(Command::Write(b)).await {
                println!("IpcWriterInstance Err writing {e}");
            }
        });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        tokio::spawn(async move {
            if let Err(e) = state::SENDER.get().unwrap().send(Command::Flush).await {
                println!("IpcWriterInstance Err flushing {e:?}");
            }
        });

        Ok(())
    }
}
