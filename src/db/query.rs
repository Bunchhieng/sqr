use crate::db::error::format_sql_error;
use crate::types::{QueryResult, Value};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::time::Instant;

/// Execute a SQL query and return results
pub fn execute_query(
    conn: &Connection,
    query: &str,
    max_rows: Option<usize>,
) -> Result<QueryResult> {
    let start = Instant::now();

    let mut stmt = conn.prepare(query).map_err(|e| {
        anyhow::anyhow!("{}", format_sql_error(&e, query))
    })?;

    // Get column names
    let columns: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Execute and collect rows
    let mut rows = Vec::new();
    let mut row_iter = stmt.query_map([], |row| {
        let mut values = Vec::new();
        for i in 0..row.as_ref().column_count() {
            let value: rusqlite::types::Value = row.get(i)?;
            values.push(Value::from(value));
        }
        Ok(values)
    })?;

    let mut truncated = false;
    let limit = max_rows.unwrap_or(1000);

    while let Some(row_result) = row_iter.next() {
        if rows.len() >= limit {
            truncated = true;
            break;
        }
        rows.push(row_result.context("Failed to read row")?);
    }

    let exec_ms = start.elapsed().as_millis() as u64;

    Ok(QueryResult {
        columns,
        rows,
        truncated,
        exec_ms,
    })
}

/// Get paginated rows from a table
pub fn get_table_rows(
    conn: &Connection,
    table_name: &str,
    limit: usize,
    offset: usize,
) -> Result<QueryResult> {
    let start = Instant::now();

    // Safely quote table name
    let safe_table = table_name.replace('"', "\"\"");
    let query = format!("SELECT * FROM \"{}\" LIMIT ? OFFSET ?", safe_table);

    let mut stmt = conn
        .prepare(&query)
        .with_context(|| format!("Failed to prepare query for table: {}", table_name))?;

    // Get column names
    let columns: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Execute with limit and offset
    let mut rows = Vec::new();
    let mut row_iter = stmt.query_map([limit as i64, offset as i64], |row| {
        let mut values = Vec::new();
        for i in 0..row.as_ref().column_count() {
            let value: rusqlite::types::Value = row.get(i)?;
            values.push(Value::from(value));
        }
        Ok(values)
    })?;

    while let Some(row_result) = row_iter.next() {
        rows.push(row_result.context("Failed to read row")?);
    }

    let exec_ms = start.elapsed().as_millis() as u64;

    Ok(QueryResult {
        columns,
        rows,
        truncated: false,
        exec_ms,
    })
}

/// Update a cell value in a table
/// Uses ROWID to identify the row, and column name to identify the column
pub fn update_cell(
    conn: &Connection,
    table_name: &str,
    row_index: usize, // Absolute row index (including pagination offset)
    column_name: &str,
    new_value: &str,
) -> Result<()> {
    // Safely quote identifiers
    let safe_table = table_name.replace('"', "\"\"");
    let safe_column = column_name.replace('"', "\"\"");
    
    // First, get the ROWID for the row at this index
    let rowid_query = format!("SELECT ROWID FROM \"{}\" LIMIT 1 OFFSET ?", safe_table);
    let rowid: i64 = conn
        .query_row(&rowid_query, [row_index as i64], |row| row.get(0))
        .with_context(|| format!("Failed to get ROWID for row {} in table: {}. Row may not exist.", row_index, table_name))?;
    
    // Parse the new value based on the column type
    // For simplicity, we'll try to infer the type from the value
    let sql_value = if new_value.trim().is_empty() || new_value.trim().eq_ignore_ascii_case("NULL") {
        "NULL".to_string()
    } else if new_value.parse::<i64>().is_ok() {
        new_value.to_string()
    } else if new_value.parse::<f64>().is_ok() {
        new_value.to_string()
    } else {
        // Treat as text
        format!("'{}'", new_value.replace('\'', "''"))
    };
    
    // Update the cell using ROWID
    let update_query = format!(
        "UPDATE \"{}\" SET \"{}\" = {} WHERE ROWID = ?",
        safe_table, safe_column, sql_value
    );
    
    conn.execute(&update_query, [rowid])
        .map_err(|e| {
            // Provide more helpful error messages
            let error_msg = e.to_string();
            if error_msg.contains("readonly") || error_msg.contains("read-only") || error_msg.contains("READONLY") {
                anyhow::anyhow!("Database is opened in read-only mode. Use --read-write flag to enable editing.")
            } else {
                anyhow::anyhow!("Failed to update cell in table {}: {}", table_name, e)
            }
        })?;
    
    Ok(())
}

