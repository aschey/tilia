use std::error::Error;
use std::io::{self, Stdout};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::{Future, Sink, StreamExt, TryStream};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{Frame, Terminal};
use tilia_widget::{BoxError, Bytes, BytesMut, LogView};
pub struct Console<'a> {
    logs: LogView<'a>,
}

impl<'a> Console<'a> {
    pub fn new<F, S, Fut>(make_transport: F) -> Self
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<S, BoxError>> + Send,
        S: TryStream<Ok = BytesMut> + Sink<Bytes> + Send + 'static,
        <S as futures::TryStream>::Error: std::fmt::Debug,
        <S as futures::Sink<Bytes>>::Error: std::fmt::Debug,
    {
        Self {
            logs: LogView::new(make_transport),
        }
    }

    pub fn from_log_view(log_view: LogView<'a>) -> Self {
        Self { logs: log_view }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        // create app and run it
        let res = self.run_app(&mut terminal).await;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    async fn run_app(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut event_reader = EventStream::new().fuse();
        loop {
            terminal.draw(|f| self.ui(f))?;
            tokio::select! {
                _ = self.logs.update() => {}
                maybe_event = event_reader.next() => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Event::Key(key) = event {
                                match (key.modifiers, key.code) {
                                    (_, KeyCode::Char('q') | KeyCode::Esc) |
                                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                                            return Ok(());
                                        },
                                    (_, KeyCode::Down) =>  self.logs.next(),
                                    (_, KeyCode::Up) =>  self.logs.previous(),
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                        _ => return Ok(())
                    }
                }

            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.size();
        let block = Block::default()
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        f.render_widget(block, size);
        self.logs.render(f, size);
    }
}
