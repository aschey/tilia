use std::{sync::atomic::Ordering, time::Duration};

use parity_tokio_ipc::Endpoint;
use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};
use tracing_subscriber::fmt::MakeWriter;

use crate::{
    command::Command,
    state::{self, ConnectState},
    WorkerGuard,
};

#[derive(Clone)]
pub struct Writer {
    app_name: String,
}

impl Writer {
    pub fn new(app_name: &str) -> (Self, WorkerGuard) {
        state::CONNECT_STATE.swap(ConnectState::NOT_CONNECTED, Ordering::SeqCst);
        (
            Self {
                app_name: app_name.to_owned(),
            },
            WorkerGuard,
        )
    }

    pub fn disabled() -> (Self, WorkerGuard) {
        (
            Self {
                app_name: "".to_owned(),
            },
            WorkerGuard,
        )
    }

    fn init(sock_path: String, mut rx: Receiver<Command>) -> bool {
        // need to ensure we don't panic if this is called outside of the tokio runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let handle = rt.spawn(async move {
                loop {
                    state::CONNECT_STATE.swap(ConnectState::NOT_CONNECTED, Ordering::SeqCst);
                    let mut last_connect = tokio::time::Instant::now();

                    let mut client = match Endpoint::connect(&sock_path).await {
                        Ok(client) => client,
                        Err(_) => loop {
                            tokio::select! {
                                client = Endpoint::connect(&sock_path), if tokio::time::Instant::now().duration_since(last_connect) > tokio::time::Duration::from_secs(1) => {
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

                    state::CONNECT_STATE.swap(ConnectState::CONNECTED, Ordering::SeqCst);
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
            true
        } else {
            false
        }
    }
}

impl MakeWriter<'_> for Writer {
    type Writer = Writer;

    fn make_writer(&'_ self) -> Self::Writer {
        if !*state::IS_INITIALIZED.read().unwrap() {
            let mut is_initialized = state::IS_INITIALIZED.write().unwrap();
            if !*is_initialized {
                #[cfg(unix)]
                let sock_path = format!("/tmp/{}logs.sock", self.app_name);
                #[cfg(windows)]
                let sock_path = format!("\\\\.\\pipe\\{}logs", self.app_name);

                let (tx, rx) = tokio::sync::mpsc::channel(32);
                state::SENDER.get_or_init(|| tx);

                *is_initialized = Self::init(sock_path, rx);
            }
        }
        self.clone()
    }
}

impl std::io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(sender) = state::SENDER.get() {
            let b = buf.to_owned();
            let _ = sender.try_send(Command::Write(b));
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(sender) = state::SENDER.get() {
            let _ = sender.try_send(Command::Flush);
        }

        Ok(())
    }
}
