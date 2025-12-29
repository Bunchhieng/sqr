use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle text editor input for a buffer with cursor position
/// Returns true if the event was handled, false otherwise
pub fn handle_text_editor_input(
    event: KeyEvent,
    buffer: &mut String,
    cursor_pos: &mut usize,
    supports_line_navigation: bool,
) -> bool {
    let pos = (*cursor_pos).min(buffer.len());

    match event.code {
        KeyCode::Char(c) => {
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'u' => {
                        // Ctrl+U: Clear from start of current line to cursor
                        if supports_line_navigation {
                            let line_start = buffer[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                            buffer.drain(line_start..pos);
                            *cursor_pos = line_start;
                        } else {
                            // If no line navigation, clear from buffer start
                            buffer.drain(0..pos);
                            *cursor_pos = 0;
                        }
                    }
                    'k' => {
                        // Ctrl+K: Clear from cursor to end of current line
                        if supports_line_navigation {
                            let line_end = buffer[pos..]
                                .find('\n')
                                .map(|i| pos + i)
                                .unwrap_or(buffer.len());
                            buffer.drain(pos..line_end);
                        } else {
                            // If no line navigation, clear to end of buffer
                            buffer.drain(pos..);
                        }
                    }
                    'a' => {
                        // Ctrl+A: Move to beginning
                        *cursor_pos = 0;
                    }
                    'e' => {
                        // Ctrl+E: Move to end
                        *cursor_pos = buffer.len();
                    }
                    'w' => {
                        // Ctrl+W: Delete word before cursor
                        if pos > 0 {
                            let mut new_pos = pos;
                            // Skip whitespace
                            while new_pos > 0
                                && buffer
                                    .chars()
                                    .nth(new_pos - 1)
                                    .is_some_and(|c| c.is_whitespace())
                            {
                                new_pos -= 1;
                            }
                            // Skip word characters
                            while new_pos > 0
                                && buffer
                                    .chars()
                                    .nth(new_pos - 1)
                                    .is_some_and(|c| !c.is_whitespace())
                            {
                                new_pos -= 1;
                            }
                            buffer.drain(new_pos..pos);
                            *cursor_pos = new_pos;
                        }
                    }
                    'd' => {
                        // Ctrl+D: Delete character at cursor
                        if pos < buffer.len() {
                            buffer.remove(pos);
                        }
                    }
                    _ => return false,
                }
            } else {
                // Regular character insertion
                buffer.insert(pos, c);
                *cursor_pos = pos + 1;
            }
            true
        }
        KeyCode::Backspace => {
            if pos > 0 {
                buffer.remove(pos - 1);
                *cursor_pos = pos - 1;
            }
            true
        }
        KeyCode::Delete => {
            if pos < buffer.len() {
                buffer.remove(pos);
            }
            true
        }
        KeyCode::Left => {
            if pos > 0 {
                *cursor_pos = pos - 1;
            }
            true
        }
        KeyCode::Right => {
            if pos < buffer.len() {
                *cursor_pos = pos + 1;
            }
            true
        }
        KeyCode::Home => {
            if supports_line_navigation {
                // Move to start of current line
                let line_start = buffer[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                *cursor_pos = line_start;
            } else {
                // Move to beginning of buffer
                *cursor_pos = 0;
            }
            true
        }
        KeyCode::End => {
            if supports_line_navigation {
                // Move to end of current line
                let line_end = buffer[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(buffer.len());
                *cursor_pos = line_end;
            } else {
                // Move to end of buffer
                *cursor_pos = buffer.len();
            }
            true
        }
        KeyCode::Up => {
            if supports_line_navigation {
                // Move to previous line
                if pos > 0 {
                    let line_start = buffer[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    if line_start > 0 {
                        let prev_line_start = buffer[..line_start - 1]
                            .rfind('\n')
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        let prev_line_end = buffer[prev_line_start..]
                            .find('\n')
                            .map(|i| prev_line_start + i)
                            .unwrap_or(buffer.len());
                        let col = pos - line_start;
                        *cursor_pos = (prev_line_start + col).min(prev_line_end);
                    }
                }
            }
            true
        }
        KeyCode::Down => {
            if supports_line_navigation {
                // Move to next line
                let line_start = buffer[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = buffer[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(buffer.len());
                if line_end < buffer.len() {
                    let next_line_start = line_end + 1;
                    let next_line_end = buffer[next_line_start..]
                        .find('\n')
                        .map(|i| next_line_start + i)
                        .unwrap_or(buffer.len());
                    let col = pos - line_start;
                    *cursor_pos = (next_line_start + col).min(next_line_end);
                }
            }
            true
        }
        _ => false,
    }
}
