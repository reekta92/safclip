use ratatui::{
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect) {
    let help_text = "SAFCLIP KEYBINDINGS & CONTROLS
═══════════════════════════════

Navigation:
  ← / →          Seek ±1s
  Shift+← / →     Seek ±5s
  Alt+← / →       Seek ±10s
  Home / End      Jump to start/end
  K               Snap to nearest keyframe
  Space           Play / Pause

Timeline Zoom/Pan:
  + / -           Zoom timeline in/out
  Alt+h / l       Pan timeline left/right

Mouse Interactions (Timeline):
  Left Click      Seek to clicked position
  Left Drag       Scrub through media (pauses during scrub, resumes on release)
  Scroll Wheel    Zoom timeline in/out anchored at mouse cursor
  Right Drag      Pan zoomed timeline horizontally

Segments:
  a               Set IN point
  d               Set OUT point (creates segment)
  Delete / x      Delete selected segment
  ↑ / ↓           Select prev/next segment

Export:
  e               Export segments as separate clips
  E (Shift+e)    Export merged clip

General:
  Tab             Switch between active MPRIS players
  u               Undo last change
  Ctrl+r          Redo
  ?               Toggle this help
  q               Quit
  Esc             Cancel / return to normal mode";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));

    let popup_area = centered_rect(65, 85, area);

    f.render_widget(Clear, popup_area);
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
