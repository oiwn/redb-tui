use crate::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
    Terminal,
};
use redb::Database;
use std::io;

pub struct TuiWrapper {
    tui: Tui,
}

impl TuiWrapper {
    pub fn new(db: &Database) -> Result<Self> {
        // Setup
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;

        let tui = Tui::new(db)?;

        Ok(Self { tui })
    }

    pub fn run(&mut self) -> Result<()> {
        self.tui.run()
    }
}

impl Drop for TuiWrapper {
    fn drop(&mut self) {
        // Teardown
        disable_raw_mode().expect("Could not disable raw mode");
        io::stdout()
            .execute(LeaveAlternateScreen)
            .expect("Could not leave alternate screen");
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    table_names: Vec<String>,
    list_state: ListState,
}

impl Tui {
    pub fn new(db: &Database) -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let table_names = crate::database::get_table_names(db)?;
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Self {
            terminal,
            table_names,
            list_state,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| {
                let items: Vec<ListItem> = self
                    .table_names
                    .iter()
                    .map(|name| ListItem::new(name.as_str()))
                    .collect();

                let list = List::new(items)
                    .block(
                        ratatui::widgets::Block::default()
                            .title("ReDB Tables")
                            .borders(ratatui::widgets::Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::LightGreen));

                frame.render_stateful_widget(
                    list,
                    frame.area(),
                    &mut self.list_state,
                );
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.previous(),
                    _ => {}
                }
            }
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.table_names.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.table_names.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
