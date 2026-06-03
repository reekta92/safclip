use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, state: &mut AppState, area: Rect) {
    let duration = state.timeline_state.duration;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Timeline (Zoom: +/- | Pan: Alt+h/l | Drag/Scroll: Mouse) ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if duration <= 0.0 || inner_area.height < 2 || inner_area.width == 0 {
        return;
    }

    // Save actual timeline render rect coordinates in AppState for mouse hit-testing
    state.timeline_rect = (inner_area.x, inner_area.y, inner_area.width, inner_area.height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Progress / Segment bar
            Constraint::Length(1), // Keyframe / Cursor marker row
        ])
        .split(inner_area);

    let bar_area = chunks[0];
    let marker_area = chunks[1];
    let width = bar_area.width as usize;

    // 1. Progress Bar Row
    let mut bar_spans = vec![Span::styled("─", Style::default().fg(Color::DarkGray)); width];

    // Segment color palette
    let palette = [
        Color::Red,
        Color::Blue,
        Color::Green,
        Color::Magenta,
        Color::Cyan,
    ];

    // Fill segments on progress bar
    for (i, segment) in state.segments.iter().enumerate() {
        let color = palette[i % palette.len()];
        let start_px = state.timeline_state.time_to_pixel(segment.start_seconds, width as u16) as i32;
        let end_px = state.timeline_state.time_to_pixel(segment.end_seconds, width as u16) as i32;
        
        let start_clamp = start_px.clamp(0, width as i32) as usize;
        let end_clamp = end_px.clamp(0, width as i32) as usize;

        for x in start_clamp..end_clamp {
            if x < width {
                bar_spans[x] = Span::styled("█", Style::default().fg(color));
            }
        }
    }

    // Draw cursor on progress bar
    let cursor_px = state.timeline_state.time_to_pixel(state.current_time, width as u16) as i32;
    if cursor_px >= 0 && cursor_px < width as i32 {
        bar_spans[cursor_px as usize] = Span::styled("▼", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    }

    let bar_line = Line::from(bar_spans);
    f.render_widget(Paragraph::new(bar_line), bar_area);

    // 2. Keyframe and Marker Row
    let mut marker_spans = vec![Span::raw(" "); width];

    // Draw keyframes as '^'
    if let Some(metadata) = &state.metadata {
        for &kf in &metadata.keyframes_seconds {
            let px = state.timeline_state.time_to_pixel(kf, width as u16) as i32;
            if px >= 0 && px < width as i32 {
                marker_spans[px as usize] = Span::styled("^", Style::default().fg(Color::DarkGray));
            }
        }
    }

    // Draw segment boundary markers
    for (i, segment) in state.segments.iter().enumerate() {
        let color = palette[i % palette.len()];
        let start_px = state.timeline_state.time_to_pixel(segment.start_seconds, width as u16) as i32;
        let end_px = state.timeline_state.time_to_pixel(segment.end_seconds, width as u16) as i32;

        if start_px >= 0 && start_px < width as i32 {
            marker_spans[start_px as usize] = Span::styled("[", Style::default().fg(color).add_modifier(Modifier::BOLD));
        }
        if end_px >= 0 && end_px < width as i32 {
            marker_spans[end_px as usize] = Span::styled("]", Style::default().fg(color).add_modifier(Modifier::BOLD));
        }
    }

    // Draw pending in-point as green '['
    if let Some(in_point) = state.pending_in_point {
        let px = state.timeline_state.time_to_pixel(in_point, width as u16) as i32;
        if px >= 0 && px < width as i32 {
            marker_spans[px as usize] = Span::styled("[", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        }
    }

    // Draw cursor on marker row
    if cursor_px >= 0 && cursor_px < width as i32 {
        marker_spans[cursor_px as usize] = Span::styled("▲", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    }

    let marker_line = Line::from(marker_spans);
    f.render_widget(Paragraph::new(marker_line), marker_area);
}
