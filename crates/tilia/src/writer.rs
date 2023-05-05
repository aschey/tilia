use crate::{
    server::RequestHandler,
    state::{self, HANDLE},
    WorkerGuard,
};
use background_service::BackgroundServiceManager;
use std::sync::atomic::Ordering;
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;
use tower_rpc::{
    make_service_fn,
    transport::{
        ipc::{self, OnConflict},
        CodecTransport,
    },
    LengthDelimitedCodec, Server,
};
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

    fn init(app_name: String, tx: broadcast::Sender<Vec<u8>>) -> bool {
        // need to ensure we don't panic if this is called outside of the tokio runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let service_manager = BackgroundServiceManager::new(CancellationToken::new());
            let mut context = service_manager.get_context();
            HANDLE
                .set(Mutex::new(Some(service_manager)))
                .expect("Handle already set");
            rt.spawn(async move {
                if let Ok(transport) = ipc::create_endpoint(&app_name, OnConflict::Overwrite) {
                    if let Ok(incoming) = transport.incoming() {
                        let server = Server::pipeline(
                            CodecTransport::new(incoming, LengthDelimitedCodec),
                            make_service_fn(move || RequestHandler::new(tx.clone())),
                        );

                        context
                            .add_service(server)
                            .await
                            .expect("Service manager stopped");
                    }
                }
            });
            return true;
        }
        false
    }
}

impl MakeWriter<'_> for Writer {
    type Writer = Writer;

    fn make_writer(&'_ self) -> Self::Writer {
        if !*state::IS_INITIALIZED.read().expect("Lock poisoned") {
            let mut is_initialized = state::IS_INITIALIZED.write().expect("Lock poisoned");
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
            sender.send(buf.to_owned()).ok();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
