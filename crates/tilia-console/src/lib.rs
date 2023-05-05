use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use std::{
    error::Error,
    io::{self, Stdout},
};
use tilia_widget::LogView;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};
pub struct Console<'a> {
    logs: LogView<'a>,
}

impl<'a> Console<'a> {
    pub fn new(name: String) -> Self {
        Self {
            logs: LogView::new(name),
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
                                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(()),
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

    fn ui(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let size = f.size();
        let block = Block::default()
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        f.render_widget(block, size);
        self.logs.render(f, size);
    }
}
