use crate::app::{App, Focus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render_tables(frame: &mut Frame, area: Rect, app: &App) {
    let filtered_tables = app.state.filtered_tables();
    let items: Vec<ListItem> = filtered_tables
        .iter()
        .map(|table| {
            let row_count = table
                .row_count
                .map(|c| format!(" ({})", c))
                .unwrap_or_default();
            let text = format!("{}{}", table.name, row_count);
            ListItem::new(text)
        })
        .collect();

    let title = if app.state.table_filter.is_empty() {
        "Tables"
    } else {
        "Tables (filtered)"
    };

    let (border_style, title_style) = if app.state.focus == Focus::Tables {
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
        .title(format!(" {} ", title)) // Add spacing for better visibility
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app.state.selected_table_index));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut list_state);

    // Show filter if active
    if !app.state.table_filter.is_empty() {
        let filter_text = format!("Filter: {}", app.state.table_filter);
        let filter_line = Line::from(Span::styled(
            filter_text,
            Style::default().fg(Color::Cyan),
        ));
        frame.render_widget(filter_line, Rect::new(area.x, area.y + area.height - 1, area.width, 1));
    }
}

