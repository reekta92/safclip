use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
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

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(if state.pending_in_point.is_some() { 1 } else { 0 }),
        ])
        .split(area);

    let items: Vec<ListItem> = state.segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let label = s.label.as_deref().unwrap_or("");
            let prefix = if Some(i) == state.selected_segment { "> " } else { "  " };
            let content = format!(
                "{}{} - {} {}",
                prefix,
                format_time(s.start_seconds),
                format_time(s.end_seconds),
                if !label.is_empty() { format!("[{}]", label) } else { "".to_string() }
            );
            let mut style = Style::default();
            if Some(i) == state.selected_segment {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Segments "));

    f.render_widget(list, chunks[0]);

    if let Some(in_point) = state.pending_in_point {
        let in_text = format!(" IN: {} | (press 'd' to set OUT)", format_time(in_point));
        let in_para = Paragraph::new(in_text)
            .style(Style::default().fg(Color::Green));
        f.render_widget(in_para, chunks[1]);
    }
}
