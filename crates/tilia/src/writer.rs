use crate::{
    history,
    server::RequestHandler,
    state::{self, HANDLE},
    WorkerGuard,
};
use background_service::BackgroundServiceManager;
use bytes::{Bytes, BytesMut};
use futures::{Future, Sink, Stream, TryStream};
use std::{
    error::Error,
    fmt::Debug,
    io,
    sync::{atomic::Ordering, Arc},
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tower_rpc::{make_service_fn, Server};
use tracing_subscriber::fmt::MakeWriter;

pub struct Writer<F, S, I, E, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: Stream<Item = Result<I, E>>,
    I: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
{
    sender: Option<history::Sender>,
    make_transport: Arc<F>,
}

impl<F, S, I, E, Fut> Clone for Writer<F, S, I, E, Fut>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = S> + Send,
    S: Stream<Item = Result<I, E>> + Send,
    I: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <I as Sink<Bytes>>::Error: Debug,
    <I as TryStream>::Error: Debug,
    E: Send,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            make_transport: self.make_transport.clone(),
        }
    }
}

impl<F, S, I, E, Fut> Writer<F, S, I, E, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = S> + Send,
    S: Stream<Item = Result<I, E>> + Send + 'static,
    I: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <I as Sink<Bytes>>::Error: Debug,
    <I as TryStream>::Error: Debug,
    E: Send + 'static,
{
    pub fn new(capacity: usize, make_transport: F) -> (Self, WorkerGuard) {
        let tx = history::channel(capacity);
        state::IS_ENABLED.swap(true, Ordering::SeqCst);
        (
            Self {
                make_transport: Arc::new(make_transport),
                sender: Some(tx),
            },
            WorkerGuard,
        )
    }

    pub fn disabled(make_transport: F) -> (Self, WorkerGuard) {
        (
            Self {
                make_transport: Arc::new(make_transport),
                sender: None,
            },
            WorkerGuard,
        )
    }

    pub fn init(&self) -> bool {
        let mut is_initialized = state::IS_INITIALIZED.write().expect("Lock poisoned");
        if !*is_initialized {
            *is_initialized = self.try_init();
        }
        *is_initialized
    }

    fn try_init(&self) -> bool {
        let sender = self.sender.clone().expect("Sender not initialized");

        // Ensure we don't panic if this is called outside of the tokio runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let service_manager = BackgroundServiceManager::new(CancellationToken::new());
            let mut context = service_manager.get_context();
            HANDLE
                .set(Mutex::new(Some(service_manager)))
                .expect("Handle already set");

            let make_transport = self.make_transport.clone();
            rt.spawn(async move {
                let transport = make_transport().await;

                let server = Server::pipeline(
                    transport,
                    make_service_fn(move || RequestHandler::new(sender.subscribe())),
                );

                context.add_service(server);
                Ok::<_, Box<dyn Error + Send + Sync>>(())
            });

            true
        } else {
            false
        }
    }
}

impl<F, S, I, E, Fut> MakeWriter<'_> for Writer<F, S, I, E, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = S> + Send,
    S: Stream<Item = Result<I, E>> + Send + 'static,
    I: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <I as Sink<Bytes>>::Error: Debug,
    <I as TryStream>::Error: Debug,
    E: Send + 'static,
{
    type Writer = Writer<F, S, I, E, Fut>;

    fn make_writer(&'_ self) -> Self::Writer {
        self.init();
        self.clone()
    }
}

impl<F, S, I, E, Fut> io::Write for Writer<F, S, I, E, Fut>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = S> + Send,
    S: Stream<Item = Result<I, E>> + Send,
    I: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <I as Sink<Bytes>>::Error: Debug,
    <I as TryStream>::Error: Debug,
    E: Send,
{
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
