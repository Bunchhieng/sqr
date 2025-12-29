use crate::app::App;
use crate::ui::text_editor::{render_editor_panel, render_text_editor_area};
use ratatui::{
    layout::Constraint,
    prelude::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn render_sql_editor(frame: &mut Frame, area: Rect, app: &App) {
    // SQL editor is display-only, always use gray style
    let border_style = Style::default().fg(Color::Gray);
    let title_style = Style::default().fg(Color::Gray);

    // Use shared editor panel rendering
    let chunks = render_editor_panel(
        frame,
        area,
        "SQL Editor (Enter to execute)",
        title_style,
        border_style,
        &[Constraint::Percentage(40), Constraint::Percentage(60)],
    );

    // Render text editor area using shared component
    render_text_editor_area(
        frame,
        chunks[0],
        &app.state.sql_query,
        app.state.sql_cursor_pos,
        "Enter SQL query here...",
        "Query",
        border_style,
    );

    // Results area
    if app.state.query_loading {
        let loading = Paragraph::new("Executing query...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().title("Results"));
        frame.render_widget(loading, chunks[1]);
    } else if let Some(error) = &app.state.query_error {
        let error_para = Paragraph::new(format!("Error:\n\n{}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default().title("Results"))
            .wrap(Wrap { trim: true });
        frame.render_widget(error_para, chunks[1]);
    } else if let Some(result) = &app.state.query_result {
        let result_text = format!(
            "{} rows in {}ms{}\n\n(Results displayed in main view)",
            result.rows.len(),
            result.exec_ms,
            if result.truncated { " (truncated)" } else { "" }
        );
        let result_para = Paragraph::new(result_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().title("Results"))
            .wrap(Wrap { trim: true });
        frame.render_widget(result_para, chunks[1]);
    } else {
        let empty = Paragraph::new("No results yet. Press Enter to execute.\n\nEditing shortcuts:\nCtrl+U: Clear line before cursor\nCtrl+K: Clear line after cursor\nCtrl+A/E: Move to start/end\nCtrl+W: Delete word\nCtrl+D: Delete char at cursor")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().title("Results"));
        frame.render_widget(empty, chunks[1]);
    }
}
