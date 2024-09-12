use crate::Result;
use redb::{Database, DatabaseStats, TableDefinition, TableHandle};
use std::path::PathBuf;

const USERS: TableDefinition<&str, u32> = TableDefinition::new("users");
const PRODUCTS: TableDefinition<u32, &str> = TableDefinition::new("products");

#[derive(Debug)]
pub struct DbProperties {
    pub file_size: u64,
    pub num_tables: usize,
}

pub fn create_dummy_database(path: &PathBuf) -> Result<()> {
    let db = Database::create(path)?;
    let write_txn = db.begin_write()?;

    {
        let mut table = write_txn.open_table(USERS)?;
        table.insert("Alice", &25)?;
        table.insert("Bob", &30)?;
        table.insert("Charlie", &35)?;
    }

    {
        let mut table = write_txn.open_table(PRODUCTS)?;
        table.insert(&1, "Laptop")?;
        table.insert(&2, "Phone")?;
        table.insert(&3, "Tablet")?;
        table.insert(&4, "Keyboard")?;
        table.insert(&5, "Wi-fi router")?;
        table.insert(&6, "Tree")?;
    }

    write_txn.commit()?;
    Ok(())
}

pub fn get_table_names(db: &Database) -> Result<Vec<String>> {
    let read_txn = db.begin_read()?;
    let tables = read_txn.list_tables()?;
    Ok(tables.into_iter().map(|t| t.name().to_string()).collect())
}

pub fn get_database_stats(db: &Database) -> DatabaseStats {
    let txn = db.begin_write().unwrap();
    let stats = txn.stats().unwrap();
    stats
}
