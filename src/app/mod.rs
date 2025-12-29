mod state;
mod text_editor;

use crate::worker::{Worker, WorkerMessage, WorkerResponse};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub use state::{AppState, Focus, ViewMode};
use text_editor::handle_text_editor_input;

/// Main application controller
pub struct App {
    pub state: AppState,
    worker: Worker,
    should_quit: bool,
}

impl App {
    pub fn new(worker: Worker, page_size: usize) -> Self {
        Self {
            state: AppState::new(page_size),
            worker,
            should_quit: false,
        }
    }

    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Process worker responses
    pub fn process_worker_responses(&mut self) -> Result<(), io::Error> {
        while let Ok(Some(response)) = self.worker.try_recv() {
            match response {
                WorkerResponse::TablesLoaded { tables } => {
                    self.state.tables = tables;
                    self.state.tables_loading = false;
                }
                WorkerResponse::TableRowsLoaded { result } => {
                    self.state.table_rows = Some(result);
                    self.state.rows_loading = false;
                }
                WorkerResponse::QueryExecuted { result } => {
                    self.state.query_result = Some(result);
                    self.state.query_error = None;
                    self.state.query_loading = false;
                    self.state.view_mode = ViewMode::Query;
                }
                WorkerResponse::TableInfoLoaded { info } => {
                    self.state.table_info = Some(info);
                }
                WorkerResponse::SchemaLoaded {
                    columns,
                    indexes,
                    foreign_keys,
                } => {
                    self.state.schema_columns = columns;
                    self.state.schema_indexes = indexes;
                    self.state.schema_foreign_keys = foreign_keys;
                    self.state.schema_loading = false;
                }
                WorkerResponse::DiagramLoaded { data } => {
                    self.state.diagram_data = Some(data);
                    self.state.diagram_loading = false;
                }
                WorkerResponse::CellUpdated => {
                    // Cell was successfully updated, reload table and exit edit mode
                    if let Some(table_name) = &self.state.current_table {
                        self.load_table(table_name.clone());
                    }
                    self.state.edit_mode = false;
                    self.state.editing_row = None;
                    self.state.editing_col = None;
                    self.state.edit_buffer.clear();
                    self.state.edit_cursor_pos = 0;
                    self.state.full_edit_mode = false;
                }
                WorkerResponse::Error { message } => {
                    // Set error based on what was loading
                    if self.state.query_loading {
                        self.state.query_error = Some(message);
                        self.state.query_loading = false;
                    } else if self.state.rows_loading {
                        self.state.query_error = Some(message);
                        self.state.rows_loading = false;
                    } else if self.state.tables_loading {
                        self.state.query_error = Some(message);
                        self.state.tables_loading = false;
                    } else if self.state.schema_loading {
                        self.state.query_error = Some(message);
                        self.state.schema_loading = false;
                    } else if self.state.diagram_loading {
                        self.state.query_error = Some(message);
                        self.state.diagram_loading = false;
                    } else if self.state.edit_mode {
                        // Show error in edit mode
                        self.state.query_error = Some(message);
                        // Don't exit edit mode on error, let user try again
                        // Clear the error after a delay or when user starts editing again
                    } else {
                        // Generic error - show it
                        self.state.query_error = Some(message);
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle a key event
    pub fn handle_key_event(&mut self, event: KeyEvent) -> Result<(), io::Error> {
        // Check if SQL editor is active and should capture input
        let sql_editor_active = self.state.show_sql_editor && self.state.focus == Focus::Content;
        // Check if full editor is active - it should capture all input
        let full_editor_active = self.state.full_edit_mode;
        
        match event.code {
            KeyCode::Char('q') if event.modifiers.is_empty() && !sql_editor_active && !full_editor_active => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                // Don't allow tab navigation when full editor is active
                if !full_editor_active {
                    if event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.state.prev_pane();
                    } else {
                        self.state.next_pane();
                    }
                }
            }
            KeyCode::Up => {
                // In full editor mode, Up is handled in the _ => branch for line navigation
                if !full_editor_active {
                    if self.state.edit_mode && !self.state.full_edit_mode {
                        if let Some(row) = self.state.editing_row {
                            if row > 0 {
                                self.state.editing_row = Some(row - 1);
                                if let Some(result) = &self.state.table_rows {
                                    if let Some(col) = self.state.editing_col {
                                        if let Some(row_data) = result.rows.get(row - 1) {
                                            if let Some(val) = row_data.get(col) {
                                                let full_value = val.display(10000);
                                                self.state.edit_buffer = full_value.clone();
                                                self.state.edit_cursor_pos = full_value.len();
                                                self.state.full_edit_mode = full_value.len() > 50 || full_value.contains('\n');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if self.state.focus == Focus::Tables {
                        self.state.move_up();
                    }
                }
            }
            KeyCode::Down => {
                // In full editor mode, Down is handled in the _ => branch for line navigation
                if !full_editor_active {
                    if self.state.edit_mode && !self.state.full_edit_mode {
                        if let Some(row) = self.state.editing_row {
                            if let Some(result) = &self.state.table_rows {
                                if row < result.rows.len().saturating_sub(1) {
                                    self.state.editing_row = Some(row + 1);
                                    if let Some(col) = self.state.editing_col {
                                        if let Some(row_data) = result.rows.get(row + 1) {
                                            if let Some(val) = row_data.get(col) {
                                                let full_value = val.display(10000);
                                                self.state.edit_buffer = full_value.clone();
                                                self.state.edit_cursor_pos = full_value.len();
                                                self.state.full_edit_mode = full_value.len() > 50 || full_value.contains('\n');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if self.state.focus == Focus::Tables {
                        self.state.move_down();
                    }
                }
            }
            KeyCode::Enter => {
                if self.state.full_edit_mode {
                    // In full editor panel, Enter saves (matching SQL editor behavior)
                    // Shift+Enter inserts newline for multi-line text
                    if event.modifiers.contains(KeyModifiers::SHIFT) {
                        // Shift+Enter inserts newline at cursor
                        let pos = self.state.edit_cursor_pos.min(self.state.edit_buffer.len());
                        self.state.edit_buffer.insert(pos, '\n');
                        self.state.edit_cursor_pos = pos + 1;
                    } else {
                        // Regular Enter saves
                        self.save_edited_cell();
                    }
                } else if self.state.edit_mode {
                    // Inline edit mode - Enter saves
                    self.save_edited_cell();
                } else if self.state.show_sql_editor && self.state.focus == Focus::Content {
                    // In SQL editor, Enter executes query
                    // Shift+Enter inserts newline for multi-line queries
                    if event.modifiers.contains(KeyModifiers::SHIFT) {
                        // Shift+Enter inserts newline at cursor
                        let pos = self.state.sql_cursor_pos.min(self.state.sql_query.len());
                        self.state.sql_query.insert(pos, '\n');
                        self.state.sql_cursor_pos = pos + 1;
                    } else {
                        // Regular Enter executes query
                        self.execute_query();
                    }
                } else if self.state.focus == Focus::Tables {
                    if let Some(table_name) = self.state.selected_table() {
                        let table_name = table_name.to_string();
                        if self.state.view_mode == ViewMode::Schema {
                            self.load_schema(table_name);
                        } else {
                            self.load_table(table_name);
                        }
                    }
                } else if self.state.focus == Focus::Content && self.state.view_mode == ViewMode::Rows {
                    // Enter edit mode for selected cell
                    self.enter_edit_mode();
                }
            }
            KeyCode::Char('d') if event.modifiers.is_empty() && !sql_editor_active && !full_editor_active => {
                // Open diagram from anywhere
                self.state.focus = Focus::Content;
                self.state.view_mode = ViewMode::Diagram;
                // Load diagram data if not already loaded
                if self.state.diagram_data.is_none() && !self.state.diagram_loading {
                    self.state.diagram_loading = true;
                    let _ = self.worker.send(WorkerMessage::LoadDiagram);
                }
            }
            KeyCode::Char('s') if event.modifiers.is_empty() && !sql_editor_active && !full_editor_active => {
                if self.state.focus == Focus::Content {
                    self.state.toggle_view_mode();
                    
                    match self.state.view_mode {
                        ViewMode::Schema => {
                            if let Some(table_name) = self.state.current_table.as_ref() {
                                self.load_schema(table_name.clone());
                            }
                        }
                        ViewMode::Diagram => {
                            // Load diagram data
                            if self.state.diagram_data.is_none() && !self.state.diagram_loading {
                                self.state.diagram_loading = true;
                                let _ = self.worker.send(WorkerMessage::LoadDiagram);
                            }
                        }
                        ViewMode::Rows => {
                            if let Some(table_name) = self.state.current_table.as_ref() {
                                self.load_table(table_name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('/') if event.modifiers.is_empty() && !sql_editor_active && !full_editor_active => {
                if self.state.focus == Focus::Tables {
                    self.state.table_filter.clear();
                }
            }
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) && sql_editor_active => {
                // Ctrl+C in SQL editor: Clear query results and reset to table view
                self.state.query_result = None;
                self.state.query_error = None;
                if self.state.view_mode == ViewMode::Query {
                    self.state.view_mode = ViewMode::Rows;
                    // Reload current table if we have one
                    if let Some(table_name) = self.state.current_table.as_ref() {
                        self.load_table(table_name.clone());
                    }
                }
            }
            KeyCode::Char('e') if event.modifiers.is_empty() && !sql_editor_active && !full_editor_active => {
                if self.state.edit_mode {
                    // Exit edit mode
                    self.state.edit_mode = false;
                    self.state.editing_row = None;
                    self.state.editing_col = None;
                    self.state.edit_buffer.clear();
                } else {
                    self.state.show_sql_editor = !self.state.show_sql_editor;
                    if !self.state.show_sql_editor {
                        self.state.sql_query.clear();
                        self.state.sql_cursor_pos = 0;
                        // Clear query results and reset view mode when closing SQL editor
                        self.state.query_result = None;
                        self.state.query_error = None;
                        if self.state.view_mode == ViewMode::Query {
                            self.state.view_mode = ViewMode::Rows;
                            if let Some(table_name) = self.state.current_table.as_ref() {
                                self.load_table(table_name.clone());
                            }
                        }
                    } else {
                        self.state.focus = Focus::Content;
                        self.state.sql_cursor_pos = self.state.sql_query.len();
                    }
                }
            }
            KeyCode::Char('?') if event.modifiers.is_empty() => {
                self.state.show_help = !self.state.show_help;
            }
            KeyCode::Left => {
                // In full editor or SQL editor mode, use text editor handler for character navigation
                if full_editor_active {
                    if handle_text_editor_input(
                        event,
                        &mut self.state.edit_buffer,
                        &mut self.state.edit_cursor_pos,
                        true,
                    ) {
                        return Ok(());
                    }
                } else if self.state.show_sql_editor && self.state.focus == Focus::Content {
                    if handle_text_editor_input(
                        event,
                        &mut self.state.sql_query,
                        &mut self.state.sql_cursor_pos,
                        true,
                    ) {
                        return Ok(());
                    }
                } else if self.state.edit_mode && !self.state.full_edit_mode {
                    if let Some(col) = self.state.editing_col {
                        if col > 0 {
                            self.state.editing_col = Some(col - 1);
                            if let Some(result) = &self.state.table_rows {
                                if let Some(row) = self.state.editing_row {
                                    if let Some(row_data) = result.rows.get(row) {
                                        if let Some(val) = row_data.get(col - 1) {
                                            self.state.edit_buffer = val.display(1000);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(());
                } else if self.state.focus == Focus::Content {
                    self.state.prev_page();
                    if let Some(table_name) = self.state.current_table.as_ref() {
                        self.load_table(table_name.clone());
                    }
                    return Ok(());
                }
            }
            KeyCode::Right => {
                // In full editor or SQL editor mode, use text editor handler for character navigation
                if full_editor_active {
                    if handle_text_editor_input(
                        event,
                        &mut self.state.edit_buffer,
                        &mut self.state.edit_cursor_pos,
                        true,
                    ) {
                        return Ok(());
                    }
                } else if self.state.show_sql_editor && self.state.focus == Focus::Content {
                    if handle_text_editor_input(
                        event,
                        &mut self.state.sql_query,
                        &mut self.state.sql_cursor_pos,
                        true,
                    ) {
                        return Ok(());
                    }
                } else if self.state.edit_mode && !self.state.full_edit_mode {
                    if let Some(col) = self.state.editing_col {
                        if let Some(result) = &self.state.table_rows {
                            if col < result.columns.len().saturating_sub(1) {
                                self.state.editing_col = Some(col + 1);
                                if let Some(row) = self.state.editing_row {
                                    if let Some(row_data) = result.rows.get(row) {
                                        if let Some(val) = row_data.get(col + 1) {
                                            self.state.edit_buffer = val.display(1000);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(());
                } else if self.state.focus == Focus::Content {
                    self.state.next_page();
                    if let Some(table_name) = self.state.current_table.as_ref() {
                        self.load_table(table_name.clone());
                    }
                    return Ok(());
                }
            }
            KeyCode::Esc => {
                if self.state.full_edit_mode {
                    // Exit full editor panel, but stay in inline edit mode
                    self.state.full_edit_mode = false;
                } else if self.state.edit_mode {
                    // Cancel edit mode completely
                    self.state.edit_mode = false;
                    self.state.editing_row = None;
                    self.state.editing_col = None;
                    self.state.edit_buffer.clear();
                    self.state.edit_cursor_pos = 0;
                    self.state.query_error = None;
                } else if self.state.show_help {
                    self.state.show_help = false;
                } else if self.state.show_sql_editor {
                    self.state.show_sql_editor = false;
                    self.state.sql_query.clear();
                    self.state.sql_cursor_pos = 0;
                    self.state.query_result = None;
                    self.state.query_error = None;
                    if self.state.view_mode == ViewMode::Query {
                        self.state.view_mode = ViewMode::Rows;
                        if let Some(table_name) = self.state.current_table.as_ref() {
                            self.load_table(table_name.clone());
                        }
                    }
                } else {
                    self.state.table_filter.clear();
                }
            }
            _ => {
                // Handle text input
                // Full editor panel captures input when active
                if self.state.full_edit_mode {
                    if let KeyCode::Char(_) = event.code {
                        self.state.query_error = None;
                    }
                    if handle_text_editor_input(
                        event,
                        &mut self.state.edit_buffer,
                        &mut self.state.edit_cursor_pos,
                        true, // supports_line_navigation
                    ) {
                        return Ok(());
                    }
                } else if self.state.edit_mode {
                    let pos = self.state.edit_cursor_pos.min(self.state.edit_buffer.len());
                    
                    match event.code {
                        KeyCode::Char(c) => {
                            self.state.query_error = None;
                            
                            if event.modifiers.contains(KeyModifiers::CONTROL) {
                                match c {
                                    'e' => {
                                        self.state.full_edit_mode = true;
                                        self.state.focus = Focus::Content;
                                        self.state.edit_cursor_pos = self.state.edit_buffer.len();
                                    }
                                    _ => {}
                                }
                            } else {
                                self.state.edit_buffer.insert(pos, c);
                                self.state.edit_cursor_pos = pos + 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if pos > 0 {
                                self.state.edit_buffer.remove(pos - 1);
                                self.state.edit_cursor_pos = pos - 1;
                            }
                        }
                        KeyCode::Delete => {
                            if pos < self.state.edit_buffer.len() {
                                self.state.edit_buffer.remove(pos);
                            }
                        }
                        KeyCode::Left => {
                            if pos > 0 {
                                self.state.edit_cursor_pos = pos - 1;
                            }
                        }
                        KeyCode::Right => {
                            if pos < self.state.edit_buffer.len() {
                                self.state.edit_cursor_pos = pos + 1;
                            }
                        }
                        KeyCode::Home => {
                            self.state.edit_cursor_pos = 0;
                        }
                        KeyCode::End => {
                            self.state.edit_cursor_pos = self.state.edit_buffer.len();
                        }
                        _ => {}
                    }
                } else if self.state.show_sql_editor && self.state.focus == Focus::Content {
                    // SQL editor input (when content pane is focused)
                    // Use shared text editor handler with line navigation support
                    if handle_text_editor_input(
                        event,
                        &mut self.state.sql_query,
                        &mut self.state.sql_cursor_pos,
                        true, // supports_line_navigation
                    ) {
                        return Ok(());
                    }
                } else if self.state.focus == Focus::Tables {
                    // Table filter input
                    if event.code == KeyCode::Char('/') {
                        self.state.table_filter.clear();
                    } else if let KeyCode::Char(c) = event.code {
                        if c != '/' {
                            self.state.table_filter.push(c);
                        }
                    } else if event.code == KeyCode::Backspace {
                        self.state.table_filter.pop();
                    }
                }
            }
        }
        Ok(())
    }

    /// Load tables from database
    pub fn load_tables(&mut self) {
        self.state.tables_loading = true;
        let _ = self.worker.send(WorkerMessage::LoadTables {
            include_internal: self.state.show_internal_tables,
        });
    }

    /// Load a specific table
    fn load_table(&mut self, table_name: String) {
        self.state.current_table = Some(table_name.clone());
        self.state.rows_loading = true;
        self.state.table_rows = None;

        let offset = self.state.current_page * self.state.page_size;
        let _ = self.worker.send(WorkerMessage::LoadTableRows {
            table_name: table_name.clone(),
            limit: self.state.page_size,
            offset,
        });

        // Also load table info
        let _ = self.worker.send(WorkerMessage::GetTableInfo {
            table_name: table_name.clone(),
        });
    }

    /// Load schema for a table
    fn load_schema(&mut self, table_name: String) {
        self.state.schema_loading = true;
        self.state.schema_columns.clear();
        self.state.schema_indexes.clear();
        self.state.schema_foreign_keys.clear();
        let _ = self.worker.send(WorkerMessage::LoadSchema {
            table_name: table_name.clone(),
        });
    }

    /// Execute SQL query
    fn execute_query(&mut self) {
        if self.state.sql_query.trim().is_empty() {
            return;
        }

        self.state.query_loading = true;
        self.state.query_error = None;
        let query = self.state.sql_query.clone();
        let _ = self.worker.send(WorkerMessage::ExecuteQuery {
            query,
            max_rows: Some(1000),
        });
    }

    /// Enter edit mode for the first cell
    fn enter_edit_mode(&mut self) {
        if let Some(result) = &self.state.table_rows {
                if !result.rows.is_empty() && !result.columns.is_empty() {
                    self.state.edit_mode = true;
                    self.state.editing_row = Some(0);
                    self.state.editing_col = Some(0);
                    if let Some(row) = result.rows.get(0) {
                        if let Some(val) = row.get(0) {
                            let full_value = val.display(10000);
                            self.state.edit_buffer = full_value.clone();
                            self.state.edit_cursor_pos = full_value.len();
                            self.state.full_edit_mode = full_value.len() > 50 || full_value.contains('\n');
                        }
                    }
                }
        }
    }

    /// Save edited cell value
    fn save_edited_cell(&mut self) {
        // Clear any previous errors
        self.state.query_error = None;
        
        if let (Some(row_idx), Some(col_idx), Some(table_name)) = (
            self.state.editing_row,
            self.state.editing_col,
            &self.state.current_table,
        ) {
            if let Some(result) = &self.state.table_rows {
                if col_idx < result.columns.len() {
                    let column_name = result.columns[col_idx].clone();
                    let new_value = self.state.edit_buffer.clone();
                    let actual_row_index = self.state.current_page * self.state.page_size + row_idx;
                    
                    if let Err(e) = self.worker.send(WorkerMessage::UpdateCell {
                        table_name: table_name.clone(),
                        row_index: actual_row_index,
                        column_name,
                        new_value,
                    }) {
                        self.state.query_error = Some(format!("Failed to send update request: {}", e));
                    }
                } else {
                    self.state.query_error = Some("Invalid column index".to_string());
                }
            } else {
                self.state.query_error = Some("No table data available".to_string());
            }
        } else {
            self.state.query_error = Some("Invalid edit state: missing row, column, or table name".to_string());
        }
    }

    /// Shutdown the application
    pub fn shutdown(self) -> Result<(), io::Error> {
        self.worker.shutdown().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to shutdown worker: {}", e))
        })
    }
}

