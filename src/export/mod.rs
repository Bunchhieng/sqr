mod csv;
mod json;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub use csv::export_csv;
pub use json::export_json;

/// Export format
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Export data to a file
pub fn export(
    conn: &Connection,
    format: ExportFormat,
    output_path: &Path,
    table_name: Option<&str>,
    query: Option<&str>,
) -> Result<()> {
    match (table_name, query) {
        (Some(table), None) => {
            // Export table
            let query_str = format!("SELECT * FROM \"{}\"", table.replace('"', "\"\""));
            export_query(conn, format, output_path, &query_str)
        }
        (None, Some(q)) => {
            // Export query results
            export_query(conn, format, output_path, q)
        }
        _ => Err(anyhow::anyhow!("Must specify either --table or --query")),
    }
}

fn export_query(
    conn: &Connection,
    format: ExportFormat,
    output_path: &Path,
    query: &str,
) -> Result<()> {
    match format {
        ExportFormat::Csv => export_csv(conn, output_path, query),
        ExportFormat::Json => export_json(conn, output_path, query),
    }
}

