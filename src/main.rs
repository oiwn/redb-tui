use std::path::PathBuf;

use clap::Parser;
use redb::Database;
use thiserror::Error;

mod database;
mod tui;

use tui::TuiWrapper;

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

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.database_path.exists() {
        database::create_dummy_database(&args.database_path)?;
        println!("Created dummy database at {:?}", args.database_path);
    }

    let db = Database::open(&args.database_path)?;
    let mut tui = TuiWrapper::new(&db)?;

    tui.run()
}
