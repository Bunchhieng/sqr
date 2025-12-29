use crate::app::{App, Focus, ViewMode};
use crate::ui::diagram::render_diagram;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let (border_style, title_style) = if app.state.focus == Focus::Content {
        (
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        (Style::default().fg(Color::Gray), Style::default().fg(Color::Gray))
    };

    let title = match app.state.view_mode {
        ViewMode::Rows => " Content ",
        ViewMode::Schema => " Schema ",
        ViewMode::Query => " Query Results ",
        ViewMode::Diagram => " ER Diagram ",
    };

    let block = Block::default()
        .title(title)
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    match app.state.view_mode {
        ViewMode::Rows => render_rows(frame, area, app, block.clone()),
        ViewMode::Schema => render_schema(frame, area, app, block.clone()),
        ViewMode::Query => render_query_results(frame, area, app, block.clone()),
        ViewMode::Diagram => render_diagram(frame, area, app, block.clone()),
    }
}

fn render_rows(frame: &mut Frame, area: Rect, app: &App, block: Block) {
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.state.rows_loading {
        let loading = Paragraph::new("Loading...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default());
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(result) = &app.state.table_rows {
        if result.columns.is_empty() {
            let empty = Paragraph::new("No columns")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default());
            frame.render_widget(empty, inner);
            return;
        }

        // Calculate column widths (equal distribution)
        let col_count = result.columns.len().max(1);
        
        // Build table rows
        let header: Vec<Cell> = result
            .columns
            .iter()
            .map(|col| {
                Cell::from(col.as_str()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();

        // Calculate max width per column (accounting for spacing)
        let max_width = (inner.width as usize / col_count).saturating_sub(2).min(50);
        
        let rows: Vec<Row> = result
            .rows
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
                let cells: Vec<Cell> = row
                    .iter()
                    .enumerate()
                    .map(|(col_idx, val)| {
                        let is_editing = app.state.edit_mode
                            && app.state.editing_row == Some(row_idx)
                            && app.state.editing_col == Some(col_idx);
                        
                        let display = if is_editing {
                            // Show edit buffer
                            if app.state.edit_buffer.is_empty() {
                                val.display(max_width)
                            } else {
                                // Truncate edit buffer if too long for display
                                let buf = &app.state.edit_buffer;
                                if buf.len() > max_width {
                                    format!("{}...", &buf[..max_width.saturating_sub(3)])
                                } else {
                                    buf.clone()
                                }
                            }
                        } else {
                            val.display(max_width)
                        };
                        
                        let mut cell = Cell::from(display);
                        if is_editing {
                            // Highlight editing cell
                            cell = cell.style(
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            );
                        }
                        cell
                    })
                    .collect();
                Row::new(cells)
            })
            .collect();
        let widths: Vec<Constraint> = (0..col_count)
            .map(|_| Constraint::Percentage((100 / col_count as u16).max(1)))
            .collect();

        let header_row = Row::new(header)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        
        let table = Table::new(rows, widths.as_slice())
            .header(header_row)
            .block(Block::default())
            .column_spacing(1)
            .widths(widths.as_slice())
            .style(Style::default().fg(Color::White));

        frame.render_widget(table, inner);

        // Show page info or edit mode hint
        let info_text = if app.state.edit_mode {
            if app.state.full_edit_mode {
                "FULL EDIT MODE - Press Enter to save, Shift+Enter for newline, Esc to exit full editor".to_string()
            } else if let Some(error) = &app.state.query_error {
                format!(
                    "ERROR: {} | Esc: Cancel | Ctrl+E: Full editor",
                    error
                )
            } else {
                format!(
                    "EDIT MODE - Row {}, Col {} | Enter: Save | Esc: Cancel | Ctrl+E: Full editor",
                    app.state.editing_row.map(|r| r + 1).unwrap_or(0),
                    app.state.editing_col.map(|c| c + 1).unwrap_or(0),
                )
            }
        } else {
            let total_rows = app.state.table_info
                .as_ref()
                .and_then(|ti| ti.row_count)
                .map(|r| format!(" of {}", r))
                .unwrap_or_default();
            format!(
                "Page {} (showing {} rows{}) - Use Left/Right to navigate | Enter: Edit cell",
                app.state.current_page + 1,
                result.rows.len(),
                total_rows
            )
        };
        let info_line = Line::from(Span::styled(
            info_text,
            Style::default().fg(if app.state.edit_mode {
                if app.state.query_error.is_some() {
                    Color::Red
                } else {
                    Color::Yellow
                }
            } else {
                Color::Gray
            }),
        ));
        frame.render_widget(
            info_line,
            Rect::new(area.x, area.y + area.height - 1, area.width, 1),
        );
    } else if let Some(table_name) = &app.state.current_table {
        let empty = Paragraph::new(format!("No data for table: {}", table_name))
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    } else {
        let empty = Paragraph::new("Select a table to view rows")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    }
}

fn render_schema(frame: &mut Frame, area: Rect, app: &App, block: Block) {
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.state.schema_loading {
        let loading = Paragraph::new("Loading schema...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default());
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(table_name) = &app.state.current_table {
        let mut lines = Vec::new();

        // Columns
        lines.push(Line::from(Span::styled(
            format!("Table: {}", table_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Columns:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        if app.state.schema_columns.is_empty() {
            lines.push(Line::from("  (no columns)"));
        } else {
            for col in &app.state.schema_columns {
                let mut col_text = format!("  {}", col.name);
                col_text.push_str(&format!(" ({})", col.data_type));
                if col.primary_key {
                    col_text.push_str(" PRIMARY KEY");
                }
                if col.not_null {
                    col_text.push_str(" NOT NULL");
                }
                if let Some(default) = &col.default_value {
                    col_text.push_str(&format!(" DEFAULT {}", default));
                }
                lines.push(Line::from(Span::styled(
                    col_text,
                    Style::default().fg(Color::White),
                )));
            }
        }

        // Indexes
        if !app.state.schema_indexes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Indexes:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for idx in &app.state.schema_indexes {
                let idx_text = format!(
                    "  {} ({})",
                    idx.name,
                    idx.columns.join(", ")
                );
                lines.push(Line::from(Span::styled(
                    idx_text,
                    Style::default().fg(Color::White),
                )));
            }
        }

        // Foreign keys
        if !app.state.schema_foreign_keys.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Foreign Keys:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for fk in &app.state.schema_foreign_keys {
                let fk_text = format!(
                    "  {} -> {}.{}",
                    fk.from_column, fk.to_table, fk.to_column
                );
                lines.push(Line::from(Span::styled(
                    fk_text,
                    Style::default().fg(Color::White),
                )));
            }
        }

        let schema = Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true });

        frame.render_widget(schema, inner);
    } else {
        let empty = Paragraph::new("Select a table to view schema")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    }
}

fn render_query_results(frame: &mut Frame, area: Rect, app: &App, block: Block) {
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.state.query_loading {
        let loading = Paragraph::new("Executing query...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default());
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(error) = &app.state.query_error {
        let error_para = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default())
            .wrap(Wrap { trim: true });
        frame.render_widget(error_para, inner);
        return;
    }

    if let Some(result) = &app.state.query_result {
        if result.columns.is_empty() {
            let empty = Paragraph::new("No columns")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default());
            frame.render_widget(empty, inner);
            return;
        }

        // Calculate column widths (equal distribution)
        let col_count = result.columns.len().max(1);
        
        // Build table rows
        let header: Vec<Cell> = result
            .columns
            .iter()
            .map(|col| {
                Cell::from(col.as_str()).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();

        // Calculate max width per column (accounting for spacing)
        let max_width = (inner.width as usize / col_count).saturating_sub(2).min(50);
        
        let rows: Vec<Row> = result
            .rows
            .iter()
            .map(|row| {
                let cells: Vec<Cell> = row
                    .iter()
                    .map(|val| {
                        let display = val.display(max_width);
                        Cell::from(display)
                    })
                    .collect();
                Row::new(cells)
            })
            .collect();
        let widths: Vec<Constraint> = (0..col_count)
            .map(|_| Constraint::Percentage((100 / col_count as u16).max(1)))
            .collect();

        let table = Table::new(rows, widths.as_slice())
            .header(Row::new(header))
            .block(Block::default())
            .column_spacing(2)
            .widths(widths.as_slice());

        frame.render_widget(table, inner);

        // Show execution info
        let info = format!(
            "{} rows in {}ms{}",
            result.rows.len(),
            result.exec_ms,
            if result.truncated { " (truncated)" } else { "" }
        );
        let info_line = Line::from(Span::styled(info, Style::default().fg(Color::Gray)));
        frame.render_widget(
            info_line,
            Rect::new(area.x, area.y + area.height - 1, area.width, 1),
        );
    } else {
        let empty = Paragraph::new("No query results")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    }
}

