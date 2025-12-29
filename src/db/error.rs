
/// User-friendly SQL error formatting
pub fn format_sql_error(error: &rusqlite::Error, query: &str) -> String {
    match error {
        rusqlite::Error::SqliteFailure(err, Some(msg)) => {
            format_sqlite_error(err.extended_code, msg, query)
        }
        rusqlite::Error::SqliteFailure(err, None) => {
            format!("SQL error (code {}): SQLite error", err.code as i32)
        }
        rusqlite::Error::InvalidColumnName(name) => {
            format!("Unknown column: '{}'\n\nHint: Check table schema with 's' key", name)
        }
        rusqlite::Error::InvalidColumnType(_, expected, actual) => {
            format!("Type mismatch: expected {}, got {}", expected, actual)
        }
        rusqlite::Error::QueryReturnedNoRows => {
            "Query returned no rows".to_string()
        }
        _ => {
            format!("SQL error: {}\n\nQuery: {}", error, truncate_query(query))
        }
    }
}

fn format_sqlite_error(code: i32, message: &str, query: &str) -> String {
    let mut result = String::new();
    
    // Common SQLite error codes with helpful messages
    match code {
        1 => { // SQLITE_ERROR
            if message.contains("no such table") {
                result.push_str("Table not found\n\n");
                result.push_str(&suggest_table_name(message, query));
            } else if message.contains("no such column") {
                result.push_str("Column not found\n\n");
                result.push_str(&suggest_column_name(message, query));
            } else {
                result.push_str(&format!("SQL error: {}\n", message));
            }
        }
        5 => { // SQLITE_BUSY
            result.push_str("Database is locked\n\n");
            result.push_str("Another process is using the database. Try again in a moment.");
        }
        19 => { // SQLITE_CONSTRAINT
            result.push_str(&format!("Constraint violation: {}\n", message));
        }
        _ => {
            result.push_str(&format!("SQL error (code {}): {}\n", code, message));
        }
    }
    
    result.push_str(&format!("\nQuery: {}", truncate_query(query)));
    result
}

fn suggest_table_name(message: &str, _query: &str) -> String {
    // Extract table name from error message if possible
    if let Some(start) = message.find(": ") {
        let table_part = &message[start + 2..];
        format!("Unknown table: {}\n\nHint: Use Tab to browse available tables", table_part)
    } else {
        "Hint: Use Tab to browse available tables".to_string()
    }
}

fn suggest_column_name(message: &str, _query: &str) -> String {
    // Extract column name from error message if possible
    if let Some(start) = message.find(": ") {
        let col_part = &message[start + 2..];
        format!("Unknown column: {}\n\nHint: Press 's' to view table schema", col_part)
    } else {
        "Hint: Press 's' to view table schema".to_string()
    }
}

fn truncate_query(query: &str) -> String {
    if query.len() > 100 {
        format!("{}...", &query[..97])
    } else {
        query.to_string()
    }
}

