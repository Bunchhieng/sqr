use crate::app::App;
use crate::ui::text_editor::{render_editor_panel, render_text_editor_area};
use ratatui::{
    layout::Constraint,
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn render_full_editor(frame: &mut Frame, area: Rect, app: &App) {
    // Get column name for title
    let column_name = if let (Some(result), Some(col_idx)) = (
        &app.state.table_rows,
        app.state.editing_col,
    ) {
        if col_idx < result.columns.len() {
            result.columns[col_idx].clone()
        } else {
            "Cell".to_string()
        }
    } else {
        "Cell".to_string()
    };

    // Editor is always focused when open, use yellow border
    let border_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let title_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    // Use shared editor panel rendering
    let chunks = render_editor_panel(
        frame,
        area,
        &format!("Full Editor: {} (Enter: Save, Shift+Enter: Newline, Esc: Cancel)", column_name),
        title_style,
        border_style,
        &[Constraint::Min(0), Constraint::Length(3)],
    );

    // Render text editor area using shared component
    render_text_editor_area(
        frame,
        chunks[0],
        &app.state.edit_buffer,
        app.state.edit_cursor_pos,
        "Enter text here...",
        "Editor",
        border_style,
    );

    // Instructions or error message
    let instructions = if let Some(error) = &app.state.query_error {
        vec![
            Line::from(vec![
                Span::styled("ERROR: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": Retry Save  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(": Cancel/Exit"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": Save  "),
                Span::styled("Shift+Enter", Style::default().fg(Color::Cyan)),
                Span::raw(": Newline  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(": Cancel/Exit  "),
                Span::styled("Ctrl+U/K", Style::default().fg(Color::Cyan)),
                Span::raw(": Clear line"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+A/E", Style::default().fg(Color::Cyan)),
                Span::raw(": Start/End  "),
                Span::styled("Ctrl+W", Style::default().fg(Color::Cyan)),
                Span::raw(": Delete word  "),
                Span::styled("Arrow keys", Style::default().fg(Color::Cyan)),
                Span::raw(": Navigate"),
            ]),
        ]
    };

    let instructions_para = Paragraph::new(instructions)
        .block(Block::default())
        .wrap(Wrap { trim: true });

    frame.render_widget(instructions_para, chunks[1]);
}

