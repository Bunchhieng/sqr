use crate::types::{ColumnInfo, DiagramData, ForeignKeyInfo, IndexInfo, QueryResult, TableInfo};

/// Current view mode in the content pane
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Rows,
    Schema,
    Query,
    Diagram,
}

/// Which pane currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tables,
    Content,
    Info,
}

/// Application state
#[derive(Debug)]
pub struct AppState {
    // Tables pane
    pub tables: Vec<TableInfo>,
    pub selected_table_index: usize,
    pub table_filter: String,
    pub show_internal_tables: bool,
    pub tables_loading: bool,

    // Content pane
    pub view_mode: ViewMode,
    pub current_table: Option<String>,
    pub table_rows: Option<QueryResult>,
    pub current_page: usize,
    pub page_size: usize,
    pub rows_loading: bool,

    // Query editor
    pub sql_query: String,
    pub query_result: Option<QueryResult>,
    pub query_error: Option<String>,
    pub query_loading: bool,

    // Info pane
    pub table_info: Option<TableInfo>,

    // Schema data
    pub schema_columns: Vec<ColumnInfo>,
    pub schema_indexes: Vec<IndexInfo>,
    pub schema_foreign_keys: Vec<ForeignKeyInfo>,
    pub schema_loading: bool,

    // Diagram data
    pub diagram_data: Option<DiagramData>,
    pub diagram_loading: bool,

    // UI state
    pub focus: Focus,
    pub show_help: bool,
    pub show_sql_editor: bool,

    // Edit mode
    pub edit_mode: bool,
    pub editing_row: Option<usize>,
    pub editing_col: Option<usize>,
    pub edit_buffer: String,
    pub edit_cursor_pos: usize,
    pub full_edit_mode: bool,
    pub sql_cursor_pos: usize,
}

impl AppState {
    pub fn new(page_size: usize) -> Self {
        Self {
            tables: Vec::new(),
            selected_table_index: 0,
            table_filter: String::new(),
            show_internal_tables: false,
            tables_loading: false,
            view_mode: ViewMode::Rows,
            current_table: None,
            table_rows: None,
            current_page: 0,
            page_size,
            rows_loading: false,
            sql_query: String::new(),
            query_result: None,
            query_error: None,
            query_loading: false,
            table_info: None,
            schema_columns: Vec::new(),
            schema_indexes: Vec::new(),
            schema_foreign_keys: Vec::new(),
            schema_loading: false,
            diagram_data: None,
            diagram_loading: false,
            focus: Focus::Content,
            show_help: false,
            show_sql_editor: true,
            edit_mode: false,
            editing_row: None,
            editing_col: None,
            edit_buffer: String::new(),
            edit_cursor_pos: 0,
            full_edit_mode: false,
            sql_cursor_pos: 0,
        }
    }

    /// Get filtered tables
    pub fn filtered_tables(&self) -> Vec<&TableInfo> {
        if self.table_filter.is_empty() {
            self.tables.iter().collect()
        } else {
            self.tables
                .iter()
                .filter(|t| {
                    t.name
                        .to_lowercase()
                        .contains(&self.table_filter.to_lowercase())
                })
                .collect()
        }
    }

    /// Get selected table name
    pub fn selected_table(&self) -> Option<&str> {
        let filtered = self.filtered_tables();
        filtered
            .get(self.selected_table_index)
            .map(|t| t.name.as_str())
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        let filtered_len = self.filtered_tables().len();
        if filtered_len > 0 {
            self.selected_table_index =
                (self.selected_table_index + filtered_len - 1) % filtered_len;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let filtered_len = self.filtered_tables().len();
        if filtered_len > 0 {
            self.selected_table_index = (self.selected_table_index + 1) % filtered_len;
        }
    }

    /// Switch to next pane (skips Info as it's informational only)
    pub fn next_pane(&mut self) {
        self.focus = match self.focus {
            Focus::Tables => Focus::Content,
            Focus::Content => Focus::Tables,
            Focus::Info => Focus::Tables,
        };
    }

    /// Switch to previous pane (skips Info as it's informational only)
    pub fn prev_pane(&mut self) {
        self.focus = match self.focus {
            Focus::Tables => Focus::Content,
            Focus::Content => Focus::Tables,
            Focus::Info => Focus::Content,
        };
    }

    /// Toggle view mode between rows and schema
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Rows => ViewMode::Schema,
            ViewMode::Schema => ViewMode::Diagram,
            ViewMode::Diagram => ViewMode::Rows,
            ViewMode::Query => ViewMode::Rows,
        };
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        self.current_page += 1;
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }
}
