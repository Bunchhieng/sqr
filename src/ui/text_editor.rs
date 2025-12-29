use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Calculate cursor position info (line, column) for display
pub fn calculate_cursor_info(text: &str, cursor_pos: usize) -> (usize, usize) {
    let pos = cursor_pos.min(text.len());
    let line = text[..pos].lines().count();
    let col = text[..pos].lines().last().map(|l| l.len()).unwrap_or(0);
    (line, col)
}

/// Render a text editor area with cursor position display
pub fn render_text_editor_area(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    cursor_pos: usize,
    placeholder: &str,
    title: &str,
    border_style: Style,
) {
    let pos = cursor_pos.min(text.chars().count());

    // Build display text with visible cursor indicator
    let display_text = if text.is_empty() {
        placeholder.to_string()
    } else {
        // Insert cursor indicator into the text at character position
        let mut display = String::new();
        for (char_idx, ch) in text.chars().enumerate() {
            if char_idx == pos {
                // Insert cursor block before this character
                display.push('█');
            }
            display.push(ch);
        }
        // If cursor is at the end, add cursor indicator
        if pos >= text.chars().count() {
            display.push('█');
        }
        display
    };

    let (line, col) = calculate_cursor_info(text, cursor_pos);
    let cursor_info = if text.is_empty() {
        title.to_string()
    } else {
        format!("{} (Line {}, Col {})", title, line, col + 1)
    };

    // Create styled text with cursor highlighted
    let mut styled_lines = Vec::new();
    for line_text in display_text.lines() {
        let mut spans = Vec::new();

        for ch in line_text.chars() {
            if ch == '█' {
                // Cursor character - highlight it
                spans.push(Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Regular character
                let style = if text.is_empty() {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                spans.push(Span::styled(ch.to_string(), style));
            }
        }
        styled_lines.push(Line::from(spans));
    }

    // If text is empty, show placeholder
    if styled_lines.is_empty() {
        styled_lines.push(Line::from(Span::styled(
            placeholder,
            Style::default().fg(Color::DarkGray),
        )));
    }

    let editor = Paragraph::new(styled_lines)
        .block(
            Block::default()
                .title(cursor_info)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(editor, area);
}

/// Render an editor panel with outer block, title, and split layout
/// Returns the inner chunks for the caller to use for editor and additional content
pub fn render_editor_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    title_style: Style,
    border_style: Style,
    editor_constraints: &[Constraint],
) -> Vec<Rect> {
    let block = Block::default()
        .title(title)
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(editor_constraints)
        .split(inner)
        .to_vec()
}
