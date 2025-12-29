use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use rusqlite::Connection;
use serde_json::{json, Value as JsonValue};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export query results to JSON
pub fn export_json(conn: &Connection, output_path: &Path, sql_query: &str) -> Result<()> {
    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

    // Execute query
    let mut stmt = conn
        .prepare(sql_query)
        .context("Failed to prepare SQL statement")?;

    // Get column names
    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    // Collect rows
    let mut rows = Vec::new();
    let row_iter = stmt.query_map([], |row| {
        let mut obj = serde_json::Map::new();
        for (i, col_name) in columns.iter().enumerate() {
            let value: rusqlite::types::Value = row.get(i)?;
            let json_value = match value {
                rusqlite::types::Value::Null => JsonValue::Null,
                rusqlite::types::Value::Integer(i) => json!(i),
                rusqlite::types::Value::Real(r) => json!(r),
                rusqlite::types::Value::Text(t) => json!(t),
                rusqlite::types::Value::Blob(b) => {
                    // Encode blob as base64
                    json!(general_purpose::STANDARD.encode(&b))
                }
            };
            obj.insert(col_name.clone(), json_value);
        }
        Ok(JsonValue::Object(obj))
    })?;

    for row_result in row_iter {
        let row = row_result.context("Failed to read row")?;
        rows.push(row);
    }

    // Write as JSON array
    let output = serde_json::to_string_pretty(&rows)
        .context("Failed to serialize JSON")?;
    file.write_all(output.as_bytes())
        .context("Failed to write JSON file")?;
    file.flush().context("Failed to flush file")?;

    Ok(())
}

