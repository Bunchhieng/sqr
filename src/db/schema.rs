use crate::types::{ColumnInfo, ForeignKeyInfo, IndexInfo, TableInfo};
use anyhow::Result;
use rusqlite::Connection;

/// Get all tables in the database
pub fn get_tables(conn: &Connection, include_internal: bool) -> Result<Vec<TableInfo>> {
    let mut stmt =
        conn.prepare("SELECT name, sql FROM sqlite_master WHERE type = 'table' ORDER BY name")?;

    let tables: Result<Vec<TableInfo>, anyhow::Error> = stmt
        .query_map([], |row| {
            Ok(TableInfo {
                name: row.get(0)?,
                row_count: None, // Will be loaded lazily
                sql: row.get(1)?,
            })
        })?
        .map(|r| r.map_err(anyhow::Error::from))
        .collect();

    let mut tables = tables?;

    if !include_internal {
        tables.retain(|t| !t.name.starts_with("sqlite_"));
    }

    // Load row counts (lazy, but do it here for now)
    for table in &mut tables {
        if let Ok(count) = get_table_row_count(conn, &table.name) {
            table.row_count = Some(count);
        }
    }

    Ok(tables)
}

/// Get row count for a table
fn get_table_row_count(conn: &Connection, table_name: &str) -> Result<u64> {
    // Use a safe query with parameter binding
    let query = format!(
        "SELECT COUNT(*) FROM \"{}\"",
        table_name.replace('"', "\"\"")
    );
    let count: i64 = conn.query_row(&query, [], |row| row.get(0))?;
    Ok(count as u64)
}

/// Get detailed information about a table
pub fn get_table_info(conn: &Connection, table_name: &str) -> Result<TableInfo> {
    let sql: Option<String> = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
        [table_name],
        |row| row.get(0),
    )?;

    let row_count = get_table_row_count(conn, table_name).ok();

    Ok(TableInfo {
        name: table_name.to_string(),
        row_count,
        sql,
    })
}

/// Get columns for a table
pub fn get_columns(conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>> {
    // Use PRAGMA table_info for reliable column information
    let mut stmt = conn.prepare(&format!(
        "PRAGMA table_info(\"{}\")",
        table_name.replace('"', "\"\"")
    ))?;

    let columns: Result<Vec<_>> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let data_type: String = row.get(2)?;
            let not_null: bool = row.get(3)?;
            let default_value: Option<String> = row.get(4)?;
            let pk: bool = row.get(5)?;

            // Check if auto-increment (heuristic: INTEGER PRIMARY KEY)
            let auto_increment = pk
                && data_type.to_uppercase().contains("INT")
                && conn
                    .query_row(
                        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
                        [table_name],
                        |row| {
                            let sql: Option<String> = row.get(0)?;
                            Ok(sql
                                .map(|s| s.to_uppercase().contains("AUTOINCREMENT"))
                                .unwrap_or(false))
                        },
                    )
                    .unwrap_or(false);

            Ok(ColumnInfo {
                name,
                data_type,
                not_null,
                default_value,
                primary_key: pk,
                auto_increment,
            })
        })?
        .map(|r| r.map_err(anyhow::Error::from))
        .collect();

    columns
}

/// Get indexes for a table
pub fn get_indexes(conn: &Connection, table_name: &str) -> Result<Vec<IndexInfo>> {
    let mut stmt = conn.prepare(
        "SELECT name, unique, sql FROM sqlite_master WHERE type = 'index' AND tbl_name = ?",
    )?;

    let indexes: Result<Vec<IndexInfo>, anyhow::Error> = stmt
        .query_map([table_name], |row| {
            let name: String = row.get(0)?;
            let unique: bool = row.get(1)?;
            let sql: Option<String> = row.get(2)?;

            // Get index columns from index_info
            let mut col_stmt = conn.prepare(&format!(
                "PRAGMA index_info(\"{}\")",
                name.replace('"', "\"\"")
            ))?;

            let columns: Result<Vec<String>, anyhow::Error> = col_stmt
                .query_map([], |row| {
                    let col_name: String = row.get(1)?;
                    Ok(col_name)
                })?
                .map(|r| r.map_err(anyhow::Error::from))
                .collect();

            let columns_result = columns.unwrap_or_default();
            Ok(IndexInfo {
                name,
                table: table_name.to_string(),
                unique,
                columns: columns_result,
                sql,
            })
        })?
        .map(|r| r.map_err(anyhow::Error::from))
        .collect();

    indexes
}

/// Get foreign keys for a table
pub fn get_foreign_keys(conn: &Connection, table_name: &str) -> Result<Vec<ForeignKeyInfo>> {
    let mut stmt = conn.prepare(&format!(
        "PRAGMA foreign_key_list(\"{}\")",
        table_name.replace('"', "\"\"")
    ))?;

    let fks: Result<Vec<ForeignKeyInfo>, anyhow::Error> = stmt
        .query_map([], |row| {
            Ok(ForeignKeyInfo {
                id: row.get(0)?,
                from_table: table_name.to_string(),
                from_column: row.get(3)?,
                to_table: row.get(2)?,
                to_column: row.get(4)?,
                on_update: row.get(5)?,
                on_delete: row.get(6)?,
            })
        })?
        .map(|r| r.map_err(anyhow::Error::from))
        .collect();

    fks
}
