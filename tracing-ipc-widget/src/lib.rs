use std::io::Stdout;

use ansi_to_tui::IntoText;
use stateful_list::StatefulList;
use tracing_ipc::run_ipc_client;
use tui::{backend::CrosstermBackend, layout::Rect, widgets::ListItem, Frame};

mod stateful_list;

pub struct LogViewBuilder {
    max_logs: usize,
    name: String,
}

impl LogViewBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            max_logs: 1000,
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
    pub fn builder(name: String) -> LogViewBuilder {
        LogViewBuilder::new(name)
    }

    fn from_builder(builder: LogViewBuilder) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            run_ipc_client(builder.name.as_ref(), tx).await;
        });

        Self {
            rx,
            logs: StatefulList::new(builder.max_logs),
            log_stream_running: true,
        }
    }

    pub fn new(name: String) -> Self {
        Self::builder(name).build()
    }

    pub async fn update(&mut self) {
        if self.log_stream_running {
            let log = self.rx.recv().await;
            if let Some(log) = log {
                let text = ListItem::new(log.into_text().expect("Invalid log"));
                self.logs.add_item(text);
            } else {
                self.log_stream_running = false;
            }
        }
    }

    pub fn next(&mut self) {
        self.logs.next();
    }

    pub fn previous(&mut self) {
        self.logs.previous();
    }

    pub fn render(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        self.logs.render(frame, area)
    }
}
