use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn render_diagram(frame: &mut Frame, area: Rect, app: &App, block: Block) {
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.state.diagram_loading {
        let loading = Paragraph::new("Loading diagram...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default());
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(diagram) = &app.state.diagram_data {
        if diagram.tables.is_empty() {
            let empty = Paragraph::new("No tables found")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default());
            frame.render_widget(empty, inner);
            return;
        }

        // Simple grid layout for tables
        // Calculate grid dimensions
        let table_count = diagram.tables.len();
        let cols = (table_count as f64).sqrt().ceil() as usize;
        let rows = (table_count + cols - 1) / cols;

        // Make tables smaller to allow arrows to cross between them
        // Add more spacing between tables for better arrow routing
        let cell_width = inner.width as usize / cols.max(1);
        let cell_height = inner.height as usize / rows.max(1);
        let spacing_x = cell_width / 3;
        let spacing_y = cell_height / 3;
        let table_width = spacing_x.max(25).min(40);
        let table_height = spacing_y.max(8).min(15);

        // Store table positions for drawing arrows
        use std::collections::HashMap;
        let mut table_positions: HashMap<String, (u16, u16, u16, u16)> = HashMap::new();

        let mut table_idx = 0;
        for row_idx in 0..rows {
            for col_idx in 0..cols {
                if table_idx >= table_count {
                    break;
                }

                let table = &diagram.tables[table_idx];
                // Add spacing between tables
                let x = inner.x + (col_idx * (table_width + spacing_x as usize)) as u16;
                let y = inner.y + (row_idx * (table_height + spacing_y as usize)) as u16;
                let available_width = (inner.width.saturating_sub(x - inner.x)) as usize;
                let available_height = (inner.height.saturating_sub(y - inner.y)) as usize;
                let width = table_width.min(available_width) as u16;
                let height = table_height.min(available_height) as u16;
                let table_area = Rect::new(x, y, width, height);

                // Store position (center x, center y, width, height)
                table_positions.insert(
                    table.name.clone(),
                    (x + width / 2, y + height / 2, width, height),
                );

                render_table_box(frame, table_area, table, diagram);
                table_idx += 1;
            }
        }

        // Draw arrows for foreign key relationships
        draw_relationship_arrows(frame, inner, diagram, &table_positions);
    } else {
        let empty = Paragraph::new("No diagram data. Press 's' to load.")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    }
}

fn render_table_box(frame: &mut Frame, area: Rect, table: &crate::types::DiagramTable, _diagram: &crate::types::DiagramData) {
    if area.width < 3 || area.height < 3 {
        return;
    }

    // Create table box with border
    let title_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    
    let block = Block::default()
        .title(table.name.as_str())
        .title_style(title_style)
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render columns (limit to fit in smaller box)
    let mut lines = Vec::new();
    let max_cols_to_show = (inner.height as usize).saturating_sub(2).min(5); // Limit columns shown
    
    for col in table.columns.iter().take(max_cols_to_show) {
        let mut spans = Vec::new();
        
        // Primary key indicator
        if col.primary_key {
            spans.push(Span::styled("*", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        } else {
            spans.push(Span::raw(" "));
        }

        // Column name (truncate if too long)
        let col_name: String = if col.name.len() > 12 {
            format!("{}...", &col.name[..12])
        } else {
            col.name.as_str().into()
        };
        
        let col_name_style = if col.primary_key {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(col_name, col_name_style));

        // Foreign key indicator (compact)
        let has_fk = table.foreign_keys.iter().any(|fk| fk.from_column == col.name);
        if has_fk {
            spans.push(Span::styled("FK", Style::default().fg(Color::Green)));
        }

        lines.push(Line::from(spans));
    }
    
    // Show indicator if more columns exist
    if table.columns.len() > max_cols_to_show {
        lines.push(Line::from(Span::styled(
            format!("... {} more", table.columns.len() - max_cols_to_show),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(lines)
        .block(Block::default())
        .wrap(Wrap { trim: true });

    frame.render_widget(para, inner);
}

fn draw_relationship_arrows(
    frame: &mut Frame,
    area: Rect,
    diagram: &crate::types::DiagramData,
    table_positions: &std::collections::HashMap<String, (u16, u16, u16, u16)>,
) {
    let buf = frame.buffer_mut();
    // Use brighter green for better visibility
    let arrow_style = Style::default().fg(Color::LightGreen);

    // Collect all table rectangles for collision detection
    let table_rects: Vec<(u16, u16, u16, u16)> = table_positions
        .values()
        .map(|&(cx, cy, w, h)| (cx - w / 2, cy - h / 2, w, h))
        .collect();

    // Collect all valid relationships and deduplicate
    // Use a set to avoid drawing the same relationship twice
    use std::collections::HashSet;
    let mut drawn_relationships: HashSet<(String, String)> = HashSet::new();
    
    // Create a set of all table names that are actually rendered
    let rendered_tables: HashSet<String> = table_positions.keys().cloned().collect();

    for table in &diagram.tables {
        for fk in &table.foreign_keys {
            // Only draw if both tables exist AND are rendered in the diagram
            if !rendered_tables.contains(&fk.from_table) 
                || !rendered_tables.contains(&fk.to_table) 
            {
                continue;
            }

            // Skip self-references (table pointing to itself)
            if fk.from_table == fk.to_table {
                continue;
            }

            // Create a canonical relationship key (always from smaller to larger table name)
            // This ensures we only draw each relationship once, regardless of direction
            let relationship_key = if fk.from_table < fk.to_table {
                (fk.from_table.clone(), fk.to_table.clone())
            } else {
                (fk.to_table.clone(), fk.from_table.clone())
            };

            // Skip if we've already drawn this relationship
            if drawn_relationships.contains(&relationship_key) {
                continue;
            }
            drawn_relationships.insert(relationship_key);

            // Get table positions - ensure both exist
            let from_pos = table_positions.get(&fk.from_table);
            let to_pos = table_positions.get(&fk.to_table);
            
            if let (Some(&(from_cx, from_cy, from_w, from_h)), Some(&(to_cx, to_cy, to_w, to_h))) = (from_pos, to_pos) {
                // Calculate connection points on table edges (not centers)
                // Start from the source table edge
                let (start_x, start_y) = find_edge_point(
                    from_cx, from_cy, from_w, from_h,
                    to_cx, to_cy
                );
                // End at the target table edge
                let (end_x, end_y) = find_edge_point(
                    to_cx, to_cy, to_w, to_h,
                    from_cx, from_cy
                );

                // Draw the arrow - the edge point calculation should ensure it connects properly
                draw_curved_arrow(buf, area, start_x, start_y, end_x, end_y, &table_rects, arrow_style);
            }
        }
    }
}

/// Find the best edge point on a table to connect from/to
fn find_edge_point(
    cx: u16, cy: u16, w: u16, h: u16,
    target_cx: u16, target_cy: u16,
) -> (u16, u16) {
    let left = cx - w / 2;
    let right = cx + w / 2;
    let top = cy - h / 2;
    let bottom = cy + h / 2;
    
    let dx = target_cx as i16 - cx as i16;
    let dy = target_cy as i16 - cy as i16;
    
    // Determine which edge to use based on direction
    if dx.abs() > dy.abs() {
        // Horizontal connection
        if dx > 0 {
            (right, cy)
        } else {
            (left, cy)
        }
    } else {
        // Vertical connection
        if dy > 0 {
            (cx, bottom)
        } else {
            (cx, top)
        }
    }
}

/// Draw a curved arrow using bezier curve approximation
fn draw_curved_arrow(
    buf: &mut Buffer,
    area: Rect,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
    table_rects: &[(u16, u16, u16, u16)],
    style: Style,
) {
    // Only draw if within bounds
    if !(x1 >= area.x && x1 < area.x + area.width
        && y1 >= area.y && y1 < area.y + area.height
        && x2 >= area.x && x2 < area.x + area.width
        && y2 >= area.y && y2 < area.y + area.height)
    {
        return;
    }

    let dx = (x2 as i16) - (x1 as i16);
    let dy = (y2 as i16) - (y1 as i16);
    let dist = ((dx * dx + dy * dy) as f64).sqrt();
    
    // Calculate control points for bezier curve
    // Use more pronounced curves for better visibility
    let control_offset = (dist * 0.5).min(25.0).max(8.0) as i16;
    
    // Create smooth S-curves that route around tables
    // Use perpendicular offsets for natural curves
    let (cx1, cy1, cx2, cy2) = if dx.abs() > dy.abs() {
        // More horizontal - create vertical curves
        let perp_offset = if dy.abs() < 5 { control_offset } else { control_offset / 2 };
        let curve_dir = if dy > 0 { 1 } else { -1 };
        (
            x1 as i16 + dx / 3,
            y1 as i16 + curve_dir * perp_offset,
            x1 as i16 + 2 * dx / 3,
            y2 as i16 - curve_dir * perp_offset,
        )
    } else {
        // More vertical - create horizontal curves
        let perp_offset = if dx.abs() < 5 { control_offset } else { control_offset / 2 };
        let curve_dir = if dx > 0 { 1 } else { -1 };
        (
            x1 as i16 + curve_dir * perp_offset,
            y1 as i16 + dy / 3,
            x2 as i16 - curve_dir * perp_offset,
            y1 as i16 + 2 * dy / 3,
        )
    };

    // Generate bezier curve points with higher resolution for smoother curves
    // Use more steps to ensure we don't skip cells
    let steps = (dist as usize * 3).max(30).min(300);
    let mut points = Vec::new();
    let mut last_point: Option<(u16, u16)> = None;
    
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let (x, y) = cubic_bezier(
            (x1 as f64, y1 as f64),
            (cx1 as f64, cy1 as f64),
            (cx2 as f64, cy2 as f64),
            (x2 as f64, y2 as f64),
            t,
        );
        let point = (x.round() as u16, y.round() as u16);
        
        // Only add point if it's different from the last one (avoid duplicates)
        if last_point != Some(point) {
            points.push(point);
            last_point = Some(point);
        }
    }
    
    // Fill in gaps between points to ensure continuous lines
    let mut filled_points = Vec::new();
    for i in 0..points.len() {
        filled_points.push(points[i]);
        if i < points.len() - 1 {
            let (px, py) = points[i];
            let (nx, ny) = points[i + 1];
            let dx = nx as i16 - px as i16;
            let dy = ny as i16 - py as i16;
            
            // Fill gaps if points are more than 1 cell apart
            if dx.abs() > 1 || dy.abs() > 1 {
                let gap_steps = dx.abs().max(dy.abs()) as usize;
                for j in 1..gap_steps {
                    let t = j as f64 / gap_steps as f64;
                    let x = (px as f64 + dx as f64 * t).round() as u16;
                    let y = (py as f64 + dy as f64 * t).round() as u16;
                    filled_points.push((x, y));
                }
            }
        }
    }
    let points = filled_points;

    // Draw the curve
    for i in 0..points.len() {
        let (x, y) = points[i];
        
        if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
            // Skip if inside a table
            let mut inside_table = false;
            for &(tx, ty, tw, th) in table_rects {
                if x >= tx && x < tx + tw && y >= ty && y < ty + th {
                    inside_table = true;
                    break;
                }
            }
            
            if !inside_table {
                let cell = buf.get_mut(x, y);
                let ch = cell.symbol().chars().next().unwrap_or(' ');
                
                if can_draw_on_cell(ch) {
                    // Determine character based on curve direction
                    let char_to_use = if i == points.len() - 1 {
                        // Arrow head at end
                        if i > 0 {
                            let (px, py) = points[i - 1];
                            let adx = x as i16 - px as i16;
                            let ady = y as i16 - py as i16;
                            if adx.abs() > ady.abs() {
                                if adx > 0 { '>' } else { '<' }
                            } else {
                                if ady > 0 { 'v' } else { '^' }
                            }
                        } else {
                            '>'
                        }
                    } else if i > 0 {
                        // Determine direction from previous point for smooth curves
                        let (px, py) = points[i - 1];
                        let adx = x as i16 - px as i16;
                        let ady = y as i16 - py as i16;
                        
                        // Use appropriate character based on direction
                        if adx.abs() > 0 && ady.abs() > 0 {
                            // Diagonal movement - use diagonal characters
                            if (adx > 0 && ady > 0) || (adx < 0 && ady < 0) { 
                                '\\' 
                            } else { 
                                '/' 
                            }
                        } else if adx.abs() > ady.abs() {
                            // Horizontal - use solid line
                            '─'
                        } else if ady.abs() > 0 {
                            // Vertical - use solid line
                            '│'
                        } else {
                            // Same position - use horizontal as default
                            '─'
                        }
                    } else {
                        // First point - use horizontal as default
                        '─'
                    };
                    
                    cell.set_char(char_to_use);
                    cell.set_style(style);
                }
            }
        }
    }
}

/// Calculate a point on a cubic bezier curve
fn cubic_bezier(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    
    let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;
    
    (x, y)
}

fn can_draw_on_cell(ch: char) -> bool {
    // Allow drawing on empty space or existing line characters
    // Also allow drawing on existing arrow characters to create overlapping paths
    ch == ' ' || ch == '─' || ch == '│' || ch == '┌' || ch == '┐' || ch == '└' || ch == '┘' 
        || ch == '├' || ch == '┤' || ch == '┬' || ch == '┴' || ch == '┼'
        || ch == '>' || ch == '<' || ch == '^' || ch == 'v'
        || ch == '/' || ch == '\\'
}

