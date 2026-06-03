use ratatui::{
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};
use crate::ui::theme::Theme;

fn key_line(key: &str, desc: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:14}", key), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(desc.to_string(), Style::default().fg(theme.muted)),
    ])
}

pub fn render(f: &mut Frame, area: Rect, theme: &Theme) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // Section: Navigation
    lines.push(Line::from(Span::styled("Navigation", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("←/→ or h/l", "Seek ±1s", theme));
    lines.push(key_line("Shift+← / →", "Seek ±5s", theme));
    lines.push(key_line("Alt+← / →", "Seek ±10s", theme));
    lines.push(key_line("Home / End", "Jump to start/end", theme));
    lines.push(key_line("K", "Snap to nearest keyframe", theme));
    lines.push(key_line("Space", "Play / Pause", theme));

    // Section: Timeline
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Timeline", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("+ / -", "Zoom timeline in/out", theme));
    lines.push(key_line("Alt+h / l", "Pan timeline left/right", theme));

    // Section: Mouse
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Mouse Interactions", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("Left Click", "Seek to clicked position", theme));
    lines.push(key_line("Left Drag", "Scrub through media", theme));
    lines.push(key_line("Scroll Wheel", "Zoom timeline at cursor", theme));
    lines.push(key_line("Right Drag", "Pan zoomed timeline", theme));

    // Section: Segments
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Segments", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("a", "Set IN point", theme));
    lines.push(key_line("d", "Set OUT point (creates segment)", theme));
    lines.push(key_line("r", "Rename selected segment", theme));
    lines.push(key_line("Delete / x / s", "Delete selected segment", theme));
    lines.push(key_line("↑/↓ or k/j", "Select prev/next segment", theme));
    lines.push(key_line("H/L or Sh+h/l", "Seek to start/end of segment", theme));

    // Section: Export
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Export", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("e", "Export segments as separate clips", theme));
    lines.push(key_line("E (Shift+e)", "Export merged clip", theme));
    lines.push(key_line("Ctrl+e", "Export selected segment only", theme));
    // Section: General
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("General", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD))));
    lines.push(key_line("Tab", "Switch active MPRIS players", theme));
    lines.push(key_line("u", "Undo last change", theme));
    lines.push(key_line("Ctrl+r", "Redo last change", theme));
    lines.push(key_line("?", "Toggle this help", theme));
    lines.push(key_line("q", "Quit", theme));
    lines.push(key_line("Esc", "Cancel / normal mode", theme));

    let block = Block::default().style(theme.popup_bg());

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.fg));

    let popup_area = centered_rect(65, 85, area);

    f.render_widget(Clear, popup_area);
    crate::ui::draw_popup_banner(f, popup_area, "HELP & CONTROLS", theme);
    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
