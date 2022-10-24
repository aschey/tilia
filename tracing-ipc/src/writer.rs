use crate::{command::Command, server::run_ipc_server, state, WorkerGuard};
use std::sync::atomic::Ordering;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct Writer {
    app_name: String,
}

impl Writer {
    pub fn new(app_name: &str) -> (Self, WorkerGuard) {
        state::IS_ENABLED.swap(true, Ordering::SeqCst);
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

    fn init(app_name: String, tx: tokio::sync::broadcast::Sender<Command>) -> bool {
        // need to ensure we don't panic if this is called outside of the tokio runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                run_ipc_server(app_name, tx).await;
            });
            return true;
        }
        false
    }
}

impl MakeWriter<'_> for Writer {
    type Writer = Writer;

    fn make_writer(&'_ self) -> Self::Writer {
        if !*state::IS_INITIALIZED.read().unwrap() {
            let mut is_initialized = state::IS_INITIALIZED.write().unwrap();
            if !*is_initialized {
                let tx = state::SENDER
                    .get_or_init(|| {
                        let (tx, _) = tokio::sync::broadcast::channel(32);
                        tx
                    })
                    .clone();

                *is_initialized = Self::init(self.app_name.to_string(), tx);
            }
        }
        self.clone()
    }
}

impl std::io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(sender) = state::SENDER.get() {
            let b = buf.to_owned();
            let _ = sender.send(Command::Write(b));
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(sender) = state::SENDER.get() {
            let _ = sender.send(Command::Flush);
        }

        Ok(())
    }
}
