use std::{fs::File, path::PathBuf};

use clap::Parser;
use log::{error, info};
use simplelog::{Config, LevelFilter, WriteLogger};
use thiserror::Error;
use tui::TuiWrapper;

mod database;
mod layout;
mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    database_path: PathBuf,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] redb::DatabaseError),
    #[error("Storage error: {0}")]
    StorageError(#[from] redb::StorageError),
    #[error("Transaction error: {0}")]
    TransactionError(#[from] redb::TransactionError),
    #[error("Table error: {0}")]
    TableError(#[from] redb::TableError),
    #[error("Commit error: {0}")]
    CommitError(#[from] redb::CommitError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, AppError>;

fn setup_logger(log_path: &PathBuf) {
    let log_file = File::create(log_path).expect("Failed to create log file");
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file)
        .expect("Failed to initialize logger");
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log_path = args.database_path.with_extension("log");
    setup_logger(&log_path);

    info!("Starting application");
    info!("Database path: {:?}", args.database_path);
    info!("Log file path: {:?}", log_path);

    if !args.database_path.exists() {
        info!("Database does not exist. Creating dummy database.");
        database::create_dummy_database(&args.database_path)?;
        info!("Created dummy database at {:?}", args.database_path);
    }

    match TuiWrapper::new(&args.database_path) {
        Ok(mut tui) => {
            info!("TUI initialized successfully.");
            if let Err(e) = tui.run() {
                error!("Error running TUI: {:?}", e);
                Err(e)
            } else {
                info!("Application finished successfully.");
                Ok(())
            }
        }
        Err(e) => {
            error!("Failed to initialize TUI: {:?}", e);
            Err(e)
        }
    }
}
