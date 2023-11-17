use ansi_to_tui::IntoText;
use futures::{Future, Sink, TryStream};
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;
use ratatui::Frame;
use stateful_list::StatefulList;
pub use tilia::{run_client, transport, BoxError, Bytes, BytesMut};
mod stateful_list;

pub struct LogViewBuilder<F, S, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync,
    Fut: Future<Output = Result<S, BoxError>> + Send,
    S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <S as futures::TryStream>::Error: std::fmt::Debug,
    <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
{
    max_logs: usize,
    make_transport: F,
}

impl<F, S, Fut> LogViewBuilder<F, S, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<S, BoxError>> + Send,
    S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
    <S as futures::TryStream>::Error: std::fmt::Debug,
    <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
{
    pub fn new(make_transport: F) -> Self {
        Self {
            make_transport,
            max_logs: 1024,
        }
    }
    pub fn with_max_logs(self, max_logs: usize) -> Self {
        Self { max_logs, ..self }
    }

    pub fn build<'a>(self) -> LogView<'a> {
        LogView::from_builder(self)
    }
}

pub struct LogView<'a> {
    rx: tokio::sync::mpsc::Receiver<String>,
    logs: StatefulList<'a>,
    log_stream_running: bool,
}

impl<'a> LogView<'a> {
    pub fn builder<F, S, Fut>(make_transport: F) -> LogViewBuilder<F, S, Fut>
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxError>> + Send,
        S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
        <S as futures::TryStream>::Error: std::fmt::Debug,
        <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
    {
        LogViewBuilder::new(make_transport)
    }

    fn from_builder<F, S, Fut>(builder: LogViewBuilder<F, S, Fut>) -> Self
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxError>> + Send,
        S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
        <S as futures::TryStream>::Error: std::fmt::Debug,
        <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            run_client(builder.make_transport, tx).await;
        });

        Self {
            rx,
            logs: StatefulList::new(builder.max_logs),
            log_stream_running: true,
        }
    }

    pub fn new<F, S, Fut>(make_transport: F) -> Self
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxError>> + Send,
        S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
        <S as futures::TryStream>::Error: std::fmt::Debug,
        <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
    {
        Self::builder(make_transport).build()
    }

    pub async fn update(&mut self) -> Result<(), ansi_to_tui::Error> {
        if self.log_stream_running {
            let log = self.rx.recv().await;
            if let Some(log) = log {
                let text = ListItem::new(log.into_text()?);
                self.logs.add_item(text);
                // Drain all pending items to prevent slow updates
                while let Ok(log) = self.rx.try_recv() {
                    let text = ListItem::new(log.into_text()?);
                    self.logs.add_item(text);
                }
            } else {
                self.log_stream_running = false;
            }
        }
        Ok(())
    }

    pub fn next(&mut self) {
        self.logs.next();
    }

    pub fn previous(&mut self) {
        self.logs.previous();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.logs.render(frame, area)
    }
}
