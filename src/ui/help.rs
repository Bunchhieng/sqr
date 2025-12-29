use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_help(frame: &mut Frame, area: Rect) {
    // Create a centered modal
    let popup_area = centered_rect(70, 80, area);

    let block = Block::default()
        .title("Help (Press ? or Esc to close)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "sqr - SQLite Explorer",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Navigation:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Tab / Shift+Tab", Style::default().fg(Color::Cyan)),
        Span::raw("  Switch between panes"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Up / Down", Style::default().fg(Color::Cyan)),
        Span::raw("  Navigate table list"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Left / Right", Style::default().fg(Color::Cyan)),
        Span::raw("  Navigate pages"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Actions:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw("  Select table / Execute SQL"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("s", Style::default().fg(Color::Cyan)),
        Span::raw("  Toggle schema ↔ rows view"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("e", Style::default().fg(Color::Cyan)),
        Span::raw("  Open SQL editor"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Ctrl+Enter", Style::default().fg(Color::Cyan)),
        Span::raw("  Execute SQL query"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("/", Style::default().fg(Color::Cyan)),
        Span::raw("  Filter tables"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::raw("  Show this help"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw("  Close modal / Clear filter"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw("  Quit application"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Panes:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from("  Left:   Table list"));
    lines.push(Line::from("  Middle: Content (rows/schema/query results)"));
    lines.push(Line::from("  Right:  Info and keybindings"));

    let para = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: true });

    frame.render_widget(para, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

