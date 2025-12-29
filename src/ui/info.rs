use crate::app::{App, Focus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Format SQL schema with syntax highlighting
fn format_sql_schema(sql: &str) -> String {
    // Basic SQL formatting: add indentation and line breaks
    let mut formatted = String::new();
    let mut indent = 0;
    let indent_size = 2;
    
    let mut chars = sql.chars().peekable();
    let mut in_string = false;
    let mut string_char = '\0';
    let mut in_comment = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' if !in_comment => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                formatted.push(ch);
            }
            '-' if !in_string && !in_comment => {
                if let Some(&'-') = chars.peek() {
                    in_comment = true;
                    formatted.push(ch);
                } else {
                    formatted.push(ch);
                }
            }
            '\n' if in_comment => {
                in_comment = false;
                formatted.push(ch);
            }
            '(' if !in_string && !in_comment => {
                formatted.push(ch);
                formatted.push('\n');
                indent += indent_size;
                formatted.push_str(&" ".repeat(indent));
            }
            ')' if !in_string && !in_comment => {
                if indent >= indent_size {
                    indent -= indent_size;
                }
                formatted.push('\n');
                formatted.push_str(&" ".repeat(indent));
                formatted.push(ch);
            }
            ',' if !in_string && !in_comment => {
                formatted.push(ch);
                formatted.push(' ');
            }
            ' ' | '\t' if !in_string && !in_comment => {
                // Collapse multiple spaces
                if !formatted.ends_with(' ') && !formatted.ends_with('\n') {
                    formatted.push(' ');
                }
            }
            _ => {
                formatted.push(ch);
            }
        }
    }
    
    formatted
}

/// Format a line of SQL with syntax highlighting
fn format_sql_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_word = String::new();
    let mut in_string = false;
    let mut string_char = '\0';
    
    // SQL keywords to highlight
    let keywords = [
        "CREATE", "TABLE", "IF", "NOT", "EXISTS", "PRIMARY", "KEY",
        "FOREIGN", "REFERENCES", "UNIQUE", "CHECK", "DEFAULT",
        "NULL", "INTEGER", "TEXT", "REAL", "BLOB", "AUTOINCREMENT",
        "CONSTRAINT", "INDEX", "ON", "DELETE", "UPDATE", "CASCADE",
        "SET", "RESTRICT", "NO", "ACTION",
    ];
    
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                    // Push current word if any
                    if !current_word.is_empty() {
                        spans.push(format_word_span(&current_word, &keywords));
                        current_word.clear();
                    }
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                } else if ch == string_char {
                    in_string = false;
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    current_word.push(ch);
                }
            }
            c if in_string => {
                spans.push(Span::styled(
                    c.to_string(),
                    Style::default().fg(Color::Green),
                ));
            }
            c if c.is_alphanumeric() || c == '_' => {
                current_word.push(c);
            }
            c => {
                // Push current word if any
                if !current_word.is_empty() {
                    spans.push(format_word_span(&current_word, &keywords));
                    current_word.clear();
                }
                // Format punctuation
                let style = match c {
                    '(' | ')' => Style::default().fg(Color::Cyan),
                    ',' => Style::default().fg(Color::Gray),
                    _ => Style::default().fg(Color::White),
                };
                spans.push(Span::styled(c.to_string(), style));
            }
        }
    }
    
    // Push remaining word
    if !current_word.is_empty() {
        spans.push(format_word_span(&current_word, &keywords));
    }
    
    if spans.is_empty() {
        Line::from("")
    } else {
        Line::from(spans)
    }
}

/// Format a word with appropriate styling based on whether it's a keyword
fn format_word_span(word: &str, keywords: &[&str]) -> Span<'static> {
    let upper_word = word.to_uppercase();
    if keywords.contains(&upper_word.as_str()) {
        Span::styled(
            word.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(word.to_string(), Style::default().fg(Color::White))
    }
}

pub fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    let (border_style, title_style) = if app.state.focus == Focus::Info {
        (
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        (Style::default().fg(Color::Gray), Style::default().fg(Color::Gray))
    };

    let block = Block::default()
        .title(" Info ")
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    if let Some(table_info) = &app.state.table_info {
        // Compact table info header
        let table_header = if let Some(row_count) = table_info.row_count {
            format!("{} ({})", table_info.name, row_count)
        } else {
            table_info.name.clone()
        };
        lines.push(Line::from(Span::styled(
            table_header,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        if let Some(sql) = &table_info.sql {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Schema:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            
            // Show only first few lines of schema to save space
            // Beautify SQL schema with syntax highlighting
            let formatted_sql = format_sql_schema(sql);
            // Collect lines as owned Strings to avoid lifetime issues
            let sql_lines: Vec<String> = formatted_sql.lines().map(|s| s.to_string()).collect();
            let max_schema_lines = 8; // Limit schema display
            let lines_to_show = sql_lines.len().min(max_schema_lines);
            
            for line in sql_lines.iter().take(lines_to_show) {
                let styled_line = format_sql_line(line);
                lines.push(styled_line);
            }
            
            // Show truncation indicator if schema is longer
            if sql_lines.len() > max_schema_lines {
                lines.push(Line::from(Span::styled(
                    format!("... ({} more lines)", sql_lines.len() - max_schema_lines),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Select a table",
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Shortcuts:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    
    // Navigation
    lines.push(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": panes  "),
        Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
        Span::raw(": nav  "),
        Span::styled("Left/Right", Style::default().fg(Color::Cyan)),
        Span::raw(": pages"),
    ]));
    
    // View modes
    lines.push(Line::from(vec![
        Span::styled("s", Style::default().fg(Color::Cyan)),
        Span::raw(": views  "),
        Span::styled("d", Style::default().fg(Color::Cyan)),
        Span::raw(": diagram  "),
        Span::styled("e", Style::default().fg(Color::Cyan)),
        Span::raw(": SQL"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::raw(": help  "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(": quit"),
    ]));
    
    // Editing shortcuts
    if app.state.edit_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Edit Mode:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        if app.state.full_edit_mode {
            lines.push(Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": save  "),
                Span::styled("Shift+Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": newline"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(": exit  "),
                Span::styled("Arrow keys", Style::default().fg(Color::Cyan)),
                Span::raw(": navigate"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Ctrl+U/K", Style::default().fg(Color::Cyan)),
                Span::raw(": clear line  "),
                Span::styled("Ctrl+A/E", Style::default().fg(Color::Cyan)),
                Span::raw(": start/end"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Ctrl+W", Style::default().fg(Color::Cyan)),
                Span::raw(": delete word"),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": save  "),
                Span::styled("Ctrl+E", Style::default().fg(Color::Cyan)),
                Span::raw(": full editor"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(": cancel  "),
                Span::styled("Left/Right", Style::default().fg(Color::Cyan)),
                Span::raw(": cells"),
            ]));
        }
    }
    
    // SQL editor shortcuts (only show if SQL editor is open)
    if app.state.show_sql_editor {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "SQL Editor:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": execute  "),
            Span::styled("Shift+Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": newline"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(": clear results  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": close"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+U/K", Style::default().fg(Color::Cyan)),
            Span::raw(": clear line  "),
            Span::styled("Ctrl+A/E", Style::default().fg(Color::Cyan)),
            Span::raw(": start/end"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+W", Style::default().fg(Color::Cyan)),
            Span::raw(": delete word  "),
            Span::styled("Arrow keys", Style::default().fg(Color::Cyan)),
            Span::raw(": navigate"),
        ]));
    }

    let para = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: true });

    frame.render_widget(para, inner);
}

