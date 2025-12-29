mod error;
pub mod query;
mod schema;

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use thiserror::Error;

pub use query::update_cell;
pub use schema::{
    get_columns, get_foreign_keys, get_indexes, get_table_info, get_tables,
};

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database file not found: {0}")]
    NotFound(String),
    #[error("Invalid SQLite file: {0}")]
    InvalidFile(String),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open a database connection
    pub fn new<P: AsRef<Path>>(path: P, read_only: bool) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Validate file exists
        if !path.as_ref().exists() {
            return Err(DatabaseError::NotFound(path_str.clone()).into());
        }

        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        };

        // Try to open the database - rusqlite will validate it's a valid SQLite file
        let conn = Connection::open_with_flags(path.as_ref(), flags)
            .with_context(|| format!("Failed to open database: {}", path_str))
            .map_err(|e| {
                // Provide more helpful error messages
                if e.to_string().contains("not a database") || e.to_string().contains("file is encrypted") {
                    DatabaseError::InvalidFile(path_str.clone()).into()
                } else {
                    anyhow::Error::from(e)
                }
            })?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        // Set busy timeout (5 seconds)
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .context("Failed to set busy timeout")?;

        Ok(Self { conn })
    }

    /// Get the underlying connection (for worker thread)
    pub fn into_connection(self) -> Connection {
        self.conn
    }

}

