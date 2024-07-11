use std::collections::VecDeque;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use ratatui::Frame;

pub(crate) struct StatefulList<'a> {
    state: ListState,
    items: VecDeque<ListItem<'a>>,
    max_logs: usize,
}

impl<'a> StatefulList<'a> {
    pub(crate) fn new(max_logs: usize) -> StatefulList<'a> {
        StatefulList {
            state: ListState::default(),
            max_logs,
            items: VecDeque::new(),
        }
    }

    pub(crate) fn add_item(&mut self, item: ListItem<'a>) {
        if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        if self.items.len() >= self.max_logs {
            self.items.pop_front();
            self.previous();
        }

        let len = self.items.len();
        self.items.push_back(item);
        if let Some(selected) = self.state.selected() {
            if len > 0 && selected == len - 1 {
                self.next();
            }
        }
    }

    pub(crate) fn next(&mut self) {
        if let Some(selected) = self.state.selected() {
            if !self.items.is_empty() && selected < self.items.len() - 1 {
                self.state.select(Some(selected + 1));
            }
        }
    }

    pub(crate) fn previous(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected > 0 {
                self.state.select(Some(selected - 1));
            }
        }
    }

    pub(crate) fn render(&mut self, frame: &mut Frame, area: Rect) {
        let logs_list = List::new(self.items.clone())
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .title("Logs"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .fg(Color::DarkGray)
                    .remove_modifier(Modifier::DIM)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(logs_list, area, &mut self.state);
    }
}
