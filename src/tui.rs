use crate::database;
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
use log::{debug, info};
use ratatui::{backend::CrosstermBackend, Terminal};
use redb::Database;
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

                let stats = database::get_database_stats(&self.db);

                let status = format!(
                    "Tables: {} | DB Size: {} Height: {} Pages: {} Stored: {} Meta: {} Frag: {}",
                    self.db_properties.num_tables,
                    self.db_properties.file_size.human_count_bytes(),
                    stats.tree_height(),
                    stats.allocated_pages(),
                    stats.stored_bytes().human_count_bytes(),
                    stats.metadata_bytes().human_count_bytes(),
                    stats.fragmented_bytes().human_count_bytes(),
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

                // NOTE: Unable to read key/values from untyped table. Typed table required
                // TableDefnition at compile time, see
                // https://github.com/cberner/redb/issues/741
                //
                // self.selected_table_content = self.read_table_content(table_name);
                // let txn = self.db.begin_read().unwrap();
                // debug!("txn: {:?}", txn);
                // let slices: TableDefinition<&[u8], &[u8]> =
                //     TableDefinition::new(&table_name);
                // debug!("slices: {:?}", slices.to_string());
                // let table = txn.open_table(slices);
                // debug!("Table: {:?}", table);
                // let table = table.unwrap();

                // // Iterate over keys; interpreting them is another challenge
                // self.selected_table_content = vec![];
                // let table_iter = table.iter();
                // debug!("Have iterator? {}", table_iter.is_err());
                // for result in table.iter().unwrap() {
                //     let (key, value) = result.unwrap();
                //     let key = String::from_utf8(key.value().to_vec())
                //         .unwrap_or("key".to_string());
                //     let value = String::from_utf8(value.value().to_vec())
                //         .unwrap_or("value".to_string());
                //     debug!("Key: {:?}, Value size: {}", key, value,);
                //     self.selected_table_content.push((key, value));
                // }

                // Fill with dummy values for now
                self.selected_table_content = vec![
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                    ("Key".to_string(), "Value".to_string()),
                ];
            }
        }
    }
}
