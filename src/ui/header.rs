use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::AppState;
use crate::player::PlayerController;
use crate::model::AppMode;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let player_name = state.active_player()
        .map(|p| p.identity())
        .unwrap_or_else(|| "No Player Connected".to_string());
    
    let title = state.active_player()
        .and_then(|p| p.track_title())
        .or_else(|| {
            state.source_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "No Media".to_string());

    let time_str = format!("{} / {}", state.timecode(), state.duration_timecode());
    
    let mode_str = match state.mode {
        AppMode::Normal => "NORMAL",
        AppMode::EditLabel => "EDIT LABEL",
        AppMode::Export => "EXPORTING",
        AppMode::Help => "HELP",
        AppMode::SessionRestore => "SESSION RESTORE",
    };

    let spans = vec![
        Span::styled(format!("[{}] ", player_name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(title, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(time_str, Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled(mode_str, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ];

    let header = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    f.render_widget(header, area);
}
