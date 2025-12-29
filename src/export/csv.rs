use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs::File;
use std::path::Path;

/// Export query results to CSV
pub fn export_csv(conn: &Connection, output_path: &Path, sql_query: &str) -> Result<()> {
    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

    let mut writer = csv::Writer::from_writer(&mut file);

    // Execute query
    let mut stmt = conn
        .prepare(sql_query)
        .context("Failed to prepare SQL statement")?;

    // Write header
    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    writer
        .write_record(&columns)
        .context("Failed to write CSV header")?;

    // Write rows
    let row_iter = stmt.query_map([], |row| {
        let mut values = Vec::new();
        for i in 0..row.as_ref().column_count() {
            let value: rusqlite::types::Value = row.get(i)?;
            let csv_value = match value {
                rusqlite::types::Value::Null => String::new(),
                rusqlite::types::Value::Integer(i) => i.to_string(),
                rusqlite::types::Value::Real(r) => r.to_string(),
                rusqlite::types::Value::Text(t) => t,
                rusqlite::types::Value::Blob(_) => "<BLOB>".to_string(),
            };
            values.push(csv_value);
        }
        Ok(values)
    })?;

    for row_result in row_iter {
        let row = row_result.context("Failed to read row")?;
        writer
            .write_record(&row)
            .context("Failed to write CSV row")?;
    }

    writer.flush().context("Failed to flush CSV writer")?;
    Ok(())
}

