mod content;
mod diagram;
mod full_editor;
mod help;
mod info;
mod sql_editor;
mod tables;
mod text_editor;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub use content::render_content;
pub use full_editor::render_full_editor;
pub use help::render_help;
pub use info::render_info;
pub use sql_editor::render_sql_editor;
pub use tables::render_tables;

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.size();

    if app.state.show_help {
        render_help(frame, size);
        return;
    }

    let has_bottom_panel = app.state.show_sql_editor || app.state.full_edit_mode;

    if has_bottom_panel {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(if app.state.full_edit_mode { 20 } else { 15 }),
            ])
            .split(size);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Tables
                Constraint::Percentage(50), // Content
                Constraint::Percentage(25), // Info
            ])
            .split(vertical_chunks[0]);

        render_tables(frame, horizontal_chunks[0], app);
        render_content(frame, horizontal_chunks[1], app);
        render_info(frame, horizontal_chunks[2], app);

        if app.state.full_edit_mode {
            render_full_editor(frame, vertical_chunks[1], app);
        } else if app.state.show_sql_editor {
            render_sql_editor(frame, vertical_chunks[1], app);
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(size);

        render_tables(frame, chunks[0], app);
        render_content(frame, chunks[1], app);
        render_info(frame, chunks[2], app);
    }
}
