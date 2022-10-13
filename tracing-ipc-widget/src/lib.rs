use std::io::Stdout;

use ansi_to_tui::IntoText;
use stateful_list::StatefulList;
use tracing_ipc::run_ipc_server;
use tui::{backend::CrosstermBackend, layout::Rect, widgets::ListItem, Frame};

mod stateful_list;

pub struct LogView<'a> {
    rx: tokio::sync::mpsc::Receiver<String>,
    logs: StatefulList<'a>,
    log_stream_running: bool,
}

impl<'a> LogView<'a> {
    pub fn new(name: String) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            run_ipc_server(name.as_ref(), tx).await;
        });

        Self {
            rx,
            logs: StatefulList::new(),
            log_stream_running: true,
        }
    }

    pub async fn update(&mut self) {
        if self.log_stream_running {
            let log = self.rx.recv().await;
            if let Some(log) = log {
                let text = ListItem::new(log.into_text().unwrap());
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
