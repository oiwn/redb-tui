use crate::Result;
use redb::{Database, TableDefinition, TableHandle};
use std::path::PathBuf;

const USERS: TableDefinition<&str, u32> = TableDefinition::new("users");
const PRODUCTS: TableDefinition<u32, &str> = TableDefinition::new("products");

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
    }

    write_txn.commit()?;
    Ok(())
}

pub fn get_table_names(db: &Database) -> Result<Vec<String>> {
    let read_txn = db.begin_read()?;
    let tables = read_txn.list_tables()?;
    Ok(tables.into_iter().map(|t| t.name().to_string()).collect())
}
