use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Padding, Paragraph},
    Frame,
};
use crate::app::AppState;

fn format_time(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "00:00.000".to_string();
    }
    let minutes = (seconds / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    let millis = ((seconds % 1.0) * 1000.0).floor() as u64;
    format!("{:02}:{:02}.{:03}", minutes, secs, millis)
}

use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, state: &mut AppState, area: Rect, theme: &Theme) {
    f.render_widget(Block::default().style(theme.segments_bg()), area);
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // List
            Constraint::Length(if state.pending_in_point.is_some() { 1 } else { 0 }), // In point
        ])
        .split(area);
    // Save actual segments list coordinates in AppState for mouse hit-testing
    state.segments_rect = (chunks[1].x, chunks[1].y, chunks[1].width, chunks[1].height);
    let header_para = Paragraph::new(Line::from(Span::styled(
        "Segments",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().style(theme.segments_bg()).padding(Padding::new(2, 2, 0, 0)));
    f.render_widget(header_para, chunks[0]);
    let items: Vec<ListItem> = state.segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let label = s.label.as_deref().unwrap_or("");
            let is_selected = Some(i) == state.selected_segment;
            let palette = theme.segment_palette();
            let seg_color = palette[i % palette.len()];
            let marker_span = if is_selected {
                Span::styled("▸ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("· ", Style::default().fg(theme.muted))
            };
            let text_style = if is_selected {
                Style::default().fg(seg_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(seg_color)
            };
            let content = format!(
                "{} - {} {}",
                format_time(s.start_seconds),
                format_time(s.end_seconds),
                if !label.is_empty() { format!("[{}]", label) } else { "".to_string() }
            );
            ListItem::new(Line::from(vec![marker_span, Span::styled(content, text_style)]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().style(theme.segments_bg()).padding(Padding::new(2, 2, 0, 0)));

    f.render_widget(list, chunks[1]);

    if let Some(in_point) = state.pending_in_point {
        use ratatui::text::Span;
        let in_line = Line::from(vec![
            Span::styled("● ", Style::default().fg(theme.success)),
            Span::styled("IN: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format_time(in_point), Style::default().fg(theme.fg)),
            Span::styled("  ·  (press 'd' to set OUT)", Style::default().fg(theme.muted)),
        ]);
        let in_para = Paragraph::new(in_line)
            .block(Block::default().style(theme.segments_bg()).padding(Padding::new(2, 2, 0, 0)));
        f.render_widget(in_para, chunks[2]);
    }
}
