use crate::database::DbProperties;
use crate::layout;
use crate::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use human_repr::HumanCount;
use log::{debug, error, info};
use ratatui::{backend::CrosstermBackend, Terminal};
use redb::{Database, DatabaseStats, ReadableTable, TableDefinition};
use std::{fs, io, path::Path};

pub struct TuiWrapper {
    tui: Tui,
}

impl TuiWrapper {
    pub fn new(db_path: &Path) -> Result<Self> {
        info!("Initializing TuiWrapper, enter alternate screen and raw mode...");
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let tui = Tui::new(db_path)?;
        Ok(Self { tui })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting TuiWrapper run loop");
        self.tui.run()
    }
}

impl Drop for TuiWrapper {
    fn drop(&mut self) {
        info!("Cleaning up TuiWrapper, exit alternate screen and raw mode...");
        disable_raw_mode().expect("Could not disable raw mode");
        io::stdout()
            .execute(LeaveAlternateScreen)
            .expect("Could not leave alternate screen");
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    table_names: Vec<String>,
    list_state: ratatui::widgets::ListState,
    db_properties: DbProperties,
    selected_table_content: Vec<(String, String)>,
    db: Database,
}

impl Tui {
    pub fn new(db_path: &Path) -> Result<Self> {
        info!("Initializing Tui with database at {:?}", db_path);
        let db = Database::open(db_path)?;
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let table_names = crate::database::get_table_names(&db)?;
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(0));

        let db_properties = DbProperties {
            file_size: fs::metadata(db_path)?.len(),
            num_tables: table_names.len(),
        };

        info!("Tui initialized successfully");
        debug!("Database properties: {:?}", db_properties);

        Ok(Self {
            db,
            terminal,
            table_names,
            list_state,
            db_properties,
            selected_table_content: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Tui run loop");
        loop {
            self.terminal.draw(|frame| {
                let size = frame.area();
                let (left, right, bottom) = layout::get_layout(size);

                layout::render_table_list(
                    frame,
                    left,
                    &self.table_names,
                    &mut self.list_state,
                );

                let binding_no_table_selected = String::from("No table selected");
                let selected_table = self
                    .table_names
                    .get(self.list_state.selected().unwrap_or(0))
                    .unwrap_or(&binding_no_table_selected);
                layout::render_key_value_pairs(
                    frame,
                    right,
                    selected_table,
                    &self.selected_table_content,
                );

                let status = format!(
                    "Tables: {} | DB Size: {}",
                    self.db_properties.num_tables,
                    self.db_properties.file_size.human_count_bytes(),
                );
                layout::render_bottom_status(frame, bottom, &status);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        info!("User requested exit");
                        return Ok(());
                    }
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.previous(),
                    _ => {}
                }
            }
        }
    }

    fn next(&mut self) {
        debug!("Moving to next item");
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
        self.update_selected_table_content();
    }

    fn previous(&mut self) {
        debug!("Moving to previous item");
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
        self.update_selected_table_content();
    }

    fn update_selected_table_content(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(table_name) = self.table_names.get(selected) {
                debug!("Updating content for selected table: {}", table_name);
                self.selected_table_content = self.read_table_content(table_name);
                self.selected_table_content =
                    vec![("Key".to_string(), "Value".to_string())];
            }
        }
    }

    fn read_table_content(&self, table_name: &str) -> Vec<(String, String)> {
        let read_txn = self.db.begin_read().unwrap();
        let table_def = TableDefinition::<&[u8], &[u8]>::new(table_name);
        let table = read_txn.open_table(table_def);

        if let Err(err) = table {
            error!("Error opening table to read content: {}", err);
            return vec![("Nothing".to_string(), "There".to_string())];
        }
        let table = table.unwrap();

        let mut content = Vec::new();
        for result in table.iter().unwrap() {
            let (key, value) = result.unwrap();
            let key_str = self.guess_type(key.value());
            let value_str = self.guess_type(value.value());
            content.push((key_str, value_str));
        }

        content
    }

    fn guess_type(&self, data: &[u8]) -> String {
        if data.is_empty() {
            return "Empty".to_string();
        }

        // Try to guess the type based on the content and length
        match data.len() {
            1 => {
                if data[0] == 0 || data[0] == 1 {
                    format!(
                        "bool: {}",
                        if data[0] == 0 { "false" } else { "true" }
                    )
                } else {
                    format!("u8: {}", u8::from_le_bytes([data[0]]))
                }
            }
            2 => format!("u16: {}", u16::from_le_bytes([data[0], data[1]])),
            4 => format!(
                "u32: {}",
                u32::from_le_bytes([data[0], data[1], data[2], data[3]])
            ),
            8 => format!("u64: {}", u64::from_le_bytes(data.try_into().unwrap())),
            _ => {
                // Try to interpret as a string
                if let Ok(s) = std::str::from_utf8(data) {
                    format!("String: {}", s)
                } else {
                    format!("Raw bytes: {:?}", data)
                }
            }
        }
    }
}
