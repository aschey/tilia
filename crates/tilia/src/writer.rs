use crate::{
    history,
    server::RequestHandler,
    state::{self, HANDLE},
    WorkerGuard,
};
use background_service::BackgroundServiceManager;
use std::{io, sync::atomic::Ordering};
use tokio::sync::Mutex;
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
pub struct Writer<const CAP: usize> {
    app_name: String,
    sender: Option<history::Sender<CAP>>,
}

impl<const CAP: usize> Writer<CAP> {
    pub fn new(app_name: &str) -> (Self, WorkerGuard) {
        let tx = history::channel();
        state::IS_ENABLED.swap(true, Ordering::SeqCst);
        (
            Self {
                app_name: app_name.to_owned(),
                sender: Some(tx),
            },
            WorkerGuard,
        )
    }

    pub fn disabled() -> (Self, WorkerGuard) {
        (
            Self {
                app_name: "".to_owned(),
                sender: None,
            },
            WorkerGuard,
        )
    }

    fn init(&self) -> bool {
        let app_name = self.app_name.clone();
        let sender = self.sender.clone().expect("Sender not initialized");

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
                            make_service_fn(move || RequestHandler::new(sender.subscribe())),
                        );

                        context
                            .add_service(server)
                            .await
                            .expect("Service manager stopped");
                    }
                }
            });
            true
        } else {
            false
        }
    }
}

impl<const CAP: usize> MakeWriter<'_> for Writer<CAP> {
    type Writer = Writer<CAP>;

    fn make_writer(&'_ self) -> Self::Writer {
        if !*state::IS_INITIALIZED.read().expect("Lock poisoned") {
            let mut is_initialized = state::IS_INITIALIZED.write().expect("Lock poisoned");
            if !*is_initialized {
                *is_initialized = self.init();
            }
        }

        self.clone()
    }
}

impl<const CAP: usize> io::Write for Writer<CAP> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(sender) = self.sender.as_mut() {
            sender.send(buf.to_owned()).ok();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
