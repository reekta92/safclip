use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::player::PlayerController;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = Vec::new();

    // 1. Connection Status
    let player_active = state.active_player_index.is_some();
    let status_span = if player_active {
        Span::styled("CONNECTED", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("DISCONNECTED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    };
    lines.push(Line::from(vec![
        Span::raw("Status: "),
        status_span,
    ]));

    // 2. Playback state
    let play_state = if state.player_playing { "PLAYING" } else { "PAUSED" };
    lines.push(Line::from(vec![
        Span::raw("Playback: "),
        Span::styled(play_state, Style::default().fg(if state.player_playing { Color::Green } else { Color::Yellow })),
    ]));

    // 3. Source Path
    let source_str = state.source_path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    lines.push(Line::from(vec![
        Span::raw("File Path: "),
        Span::styled(source_str, Style::default().fg(Color::Cyan)),
    ]));

    // 4. Keyframe info
    let kf_text = if state.is_probing {
        Span::styled("Probing keyframes...", Style::default().fg(Color::Yellow))
    } else if let Some(meta) = &state.metadata {
        if meta.keyframes_seconds.is_empty() {
            Span::styled("No keyframes found", Style::default().fg(Color::Red))
        } else {
            Span::styled(format!("Loaded ({} keyframes)", meta.keyframes_seconds.len()), Style::default().fg(Color::Green))
        }
    } else {
        Span::styled("No keyframe data (Probe with local file)", Style::default().fg(Color::Gray))
    };
    lines.push(Line::from(vec![
        Span::raw("Keyframes: "),
        kf_text,
    ]));

    lines.push(Line::from(""));

    // 5. Available Players List
    lines.push(Line::from(Span::styled("Available Players:", Style::default().add_modifier(Modifier::UNDERLINED))));
    if state.available_players.is_empty() {
        lines.push(Line::from(Span::styled("  (No MPRIS players found)", Style::default().fg(Color::DarkGray))));
    } else {
        for (i, player) in state.available_players.iter().enumerate() {
            let is_active = Some(i) == state.active_player_index;
            let marker = if is_active { "● " } else { "○ " };
            let style = if is_active {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(Span::styled(
                format!("  {}{}", marker, player.identity()),
                style
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Press [Tab] to cycle players", Style::default().fg(Color::DarkGray))));

    let info_para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Player Info "));

    f.render_widget(info_para, area);
}
