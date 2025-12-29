use serde::{Deserialize, Serialize};
use crate::types::{ColumnInfo, ForeignKeyInfo};

/// Table data for ER diagram visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramTable {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

/// Complete diagram data with all tables and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramData {
    pub tables: Vec<DiagramTable>,
}

