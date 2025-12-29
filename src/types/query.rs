use rusqlite::types::Value as SqliteValue;
use serde::{Deserialize, Serialize};

/// Display-friendly value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<SqliteValue> for Value {
    fn from(v: SqliteValue) -> Self {
        match v {
            SqliteValue::Null => Value::Null,
            SqliteValue::Integer(i) => Value::Integer(i),
            SqliteValue::Real(r) => Value::Real(r),
            SqliteValue::Text(t) => Value::Text(t),
            SqliteValue::Blob(b) => Value::Blob(b),
        }
    }
}

impl Value {
    /// Format value for display, truncating long text/blob
    pub fn display(&self, max_len: usize) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Real(r) => {
                // Format with reasonable precision
                if r.fract() == 0.0 {
                    format!("{:.0}", r)
                } else {
                    format!("{:.6}", r)
                }
            }
            Value::Text(t) => {
                if t.len() > max_len {
                    format!("{}...", &t[..max_len.saturating_sub(3)])
                } else {
                    t.clone()
                }
            }
            Value::Blob(b) => {
                if b.len() > max_len {
                    format!("<BLOB {} bytes>...", b.len())
                } else {
                    format!("<BLOB {} bytes>", b.len())
                }
            }
        }
    }
}

/// Query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub truncated: bool,
    pub exec_ms: u64,
}

impl QueryResult {
    #[allow(dead_code)]
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Value>>, exec_ms: u64) -> Self {
        Self {
            columns,
            rows,
            truncated: false,
            exec_ms,
        }
    }

    #[allow(dead_code)]
    pub fn with_truncation(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }
}
