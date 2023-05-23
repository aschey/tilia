use crate::{
    history,
    server::RequestHandler,
    state::{self, HANDLE},
    TransportType, WorkerGuard,
};
use background_service::BackgroundServiceManager;
use std::{error::Error, io, sync::atomic::Ordering};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower_rpc::{
    make_service_fn,
    transport::{
        ipc::{self, OnConflict},
        tcp, CodecTransport,
    },
    IntoBoxedStream, LengthDelimitedCodec, Server,
};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct Writer<const CAP: usize> {
    transport_type: TransportType,
    sender: Option<history::Sender<CAP>>,
}

impl<const CAP: usize> Writer<CAP> {
    pub fn new(transport_type: TransportType) -> (Self, WorkerGuard) {
        let tx = history::channel();
        state::IS_ENABLED.swap(true, Ordering::SeqCst);
        (
            Self {
                transport_type,
                sender: Some(tx),
            },
            WorkerGuard,
        )
    }

    pub fn disabled() -> (Self, WorkerGuard) {
        (
            Self {
                transport_type: TransportType::Ipc("".to_owned()),
                sender: None,
            },
            WorkerGuard,
        )
    }

    fn init(&self) -> bool {
        let sender = self.sender.clone().expect("Sender not initialized");

        // need to ensure we don't panic if this is called outside of the tokio runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let service_manager = BackgroundServiceManager::new(CancellationToken::new());
            let mut context = service_manager.get_context();
            HANDLE
                .set(Mutex::new(Some(service_manager)))
                .expect("Handle already set");
            let transport_type = self.transport_type.clone();
            rt.spawn(async move {
                let transport = match transport_type {
                    TransportType::Ipc(app_name) => {
                        let transport = ipc::create_endpoint(app_name, OnConflict::Overwrite)?;
                        transport.incoming().unwrap().into_boxed()
                    }
                    TransportType::Tcp(addr) => {
                        let transport = tcp::TcpTransport::bind(addr).await?;
                        transport.into_boxed()
                    }
                };

                let server = Server::pipeline(
                    CodecTransport::new(transport, LengthDelimitedCodec),
                    make_service_fn(move || RequestHandler::new(sender.subscribe())),
                );

                context
                    .add_service(server)
                    .await
                    .expect("Service manager stopped");
                Ok::<_, Box<dyn Error + Send + Sync>>(())
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
