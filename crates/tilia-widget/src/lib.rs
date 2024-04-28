use std::error::Error;

use ansi_to_tui::IntoText;
use futures::{Future, Sink, Stream};
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;
use ratatui::Frame;
use stateful_list::StatefulList;
pub use tilia::{run_client, transport, BoxedError, Bytes, BytesMut};
mod stateful_list;

pub struct LogViewBuilder<F, S, E, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync,
    Fut: Future<Output = Result<S, BoxedError>> + Send,
    S: Stream<Item = Result<BytesMut, E>> + Sink<Bytes> + Send + Unpin + 'static,
    E: Error + Send + Sync,
{
    max_logs: usize,
    make_transport: F,
}

impl<F, S, E, Fut> LogViewBuilder<F, S, E, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<S, BoxedError>> + Send,
    S: Stream<Item = Result<BytesMut, E>> + Sink<Bytes> + Send + Unpin + 'static,
    E: Error + Send + Sync,
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
    pub fn builder<F, S, E, Fut>(make_transport: F) -> LogViewBuilder<F, S, E, Fut>
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxedError>> + Send,
        S: Stream<Item = Result<BytesMut, E>> + Sink<Bytes> + Send + Unpin + 'static,
        E: Error + Send + Sync,
    {
        LogViewBuilder::new(make_transport)
    }

    fn from_builder<F, S, E, Fut>(builder: LogViewBuilder<F, S, E, Fut>) -> Self
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxedError>> + Send,
        S: Stream<Item = Result<BytesMut, E>> + Sink<Bytes> + Send + Unpin + 'static,
        E: Error + Send + Sync,
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

    pub fn new<F, S, E, Fut>(make_transport: F) -> Self
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxedError>> + Send,
        S: Stream<Item = Result<BytesMut, E>> + Sink<Bytes> + Send + Unpin + 'static,
        E: Error + Send + Sync,
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
