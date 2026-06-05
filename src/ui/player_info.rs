use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::player::PlayerController;

use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, state: &AppState, area: Rect, theme: &Theme) {
    let mut lines = Vec::new();

    // Section header
    lines.push(Line::from(Span::styled("Player Info", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));

    // 1. Connection Status
    let player_active = state.active_player_index.is_some();
    let status_span = if player_active {
        Span::styled("◉ Connected", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("○ Disconnected", Style::default().fg(theme.destructive).add_modifier(Modifier::BOLD))
    };
    lines.push(Line::from(vec![
        Span::styled("Status:    ", Style::default().fg(theme.muted)),
        status_span,
    ]));

    // 2. Playback state
    let play_state = if state.player_playing { "PLAYING" } else { "PAUSED" };
    lines.push(Line::from(vec![
        Span::styled("Playback:  ", Style::default().fg(theme.muted)),
        Span::styled(play_state, Style::default().fg(if state.player_playing { theme.success } else { theme.heading }).add_modifier(Modifier::BOLD)),
    ]));

    // 3. Source Path
    let source_str = state.source_path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    lines.push(Line::from(vec![
        Span::styled("File Path: ", Style::default().fg(theme.muted)),
        Span::styled(source_str, Style::default().fg(theme.accent)),
    ]));

    // 4. Keyframe info
    let kf_text = if state.is_probing {
        Span::styled("Probing keyframes...", Style::default().fg(theme.heading))
    } else if let Some(meta) = &state.metadata {
        if meta.keyframes_seconds.is_empty() {
            Span::styled("No keyframes found", Style::default().fg(theme.destructive))
        } else {
            Span::styled(format!("Loaded ({} keyframes)", meta.keyframes_seconds.len()), Style::default().fg(theme.success))
        }
    } else {
        Span::styled("No keyframe data (Probe with local file)", Style::default().fg(theme.muted))
    };
    lines.push(Line::from(vec![
        Span::styled("Keyframes: ", Style::default().fg(theme.muted)),
        kf_text,
    ]));

    lines.push(Line::from(""));

    // 5. Available Players List
    lines.push(Line::from(Span::styled("Available Players:", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    if state.available_players.is_empty() {
        lines.push(Line::from(Span::styled("  (No MPRIS players found)", Style::default().fg(theme.muted))));
    } else {
        for (i, player) in state.available_players.iter().enumerate() {
            let player: &Box<dyn PlayerController> = player;
            let is_active = Some(i) == state.active_player_index;
            let marker = if is_active { "● " } else { "○ " };
            let style = if is_active {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };
            lines.push(Line::from(Span::styled(
                format!("  {}{}", marker, player.identity()),
                style
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Press [Tab] to cycle players", Style::default().fg(theme.muted))));

    let info_para = Paragraph::new(lines)
        .block(Block::default().style(theme.player_info_bg()).padding(Padding::new(2, 2, 0, 0)))
        .style(Style::default().fg(theme.fg));

    f.render_widget(info_para, area);
}
