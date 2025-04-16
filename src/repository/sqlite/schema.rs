use rusqlite::{Connection, Result};

/// Initialize the SQLite database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS expenses (
            id INTEGER PRIMARY KEY,
            amount REAL NOT NULL,
            category TEXT NOT NULL,
            category_description TEXT,
            date TEXT NOT NULL,
            description TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}
