use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::player::PlayerController;
use crate::model::AppMode;

use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, state: &AppState, area: Rect, theme: &Theme) {
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
        Span::styled("▶ ", Style::default().fg(theme.accent)),
        Span::styled(format!("{}  ", player_name), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(title, Style::default().fg(theme.fg)),
        Span::styled("  ·  ", Style::default().fg(theme.muted)),
        Span::styled(time_str, Style::default().fg(theme.muted)),
        Span::styled("  ·  ", Style::default().fg(theme.muted)),
        Span::styled(mode_str, Style::default().fg(theme.heading).add_modifier(Modifier::BOLD)),
    ];

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border))
        .style(theme.header_bg())
        .padding(Padding::new(2, 2, 0, 0));

    let header = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .block(block)
        .style(Style::default().fg(theme.fg));

    f.render_widget(header, area);
}
