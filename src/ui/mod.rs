use ratatui::{Frame, layout::{Constraint, Direction, Layout}};
use crate::app::AppState;
use crate::model::AppMode;

pub mod header;
pub mod help;
pub mod segments;
pub mod status;
pub mod timeline;
pub mod player_info;

pub fn render(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),      // Header
            Constraint::Min(10),        // Main content area
            Constraint::Length(4),      // Timeline (2 for borders, 2 for content)
            Constraint::Length(1),      // Status bar
        ])
        .split(f.area());

    header::render(f, state, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Player status & selection
            Constraint::Percentage(50), // Right: Segments list
        ])
        .split(chunks[1]);

    player_info::render(f, state, main_chunks[0]);
    segments::render(f, state, main_chunks[1]);

    timeline::render(f, state, chunks[2]);
    status::render(f, state, chunks[3]);

    if state.mode == AppMode::Help {
        help::render(f, f.area());
    }
    if state.mode == AppMode::SessionRestore {
        render_restore_popup(f, state, f.area());
    }
}

fn render_restore_popup(f: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};
    use ratatui::style::{Color, Style, Modifier};
    use ratatui::layout::Alignment;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Restore Session? ");

    let mut text = Vec::new();
    text.push(ratatui::text::Line::from(""));

    if let Some(session) = &state.pending_session {
        let seg_count = session.segments.len();
        text.push(ratatui::text::Line::from(vec![
            ratatui::text::Span::raw("A previous session with "),
            ratatui::text::Span::styled(format!("{}", seg_count), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ratatui::text::Span::raw(" segment(s) was found."),
        ]));

        if let Some(source_path) = &state.source_path {
            let validation = crate::session::validate(session, source_path);
            if !validation.modified_match {
                text.push(ratatui::text::Line::from(""));
                text.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    "⚠️ Warning: Source file modified time does not match session save time.",
                    Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
                )));
                text.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    "Keyframe positions could differ.",
                    Style::default().fg(Color::LightRed)
                )));
            }
        }
    } else {
        text.push(ratatui::text::Line::from("A previous session was found."));
    }

    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from(vec![
        ratatui::text::Span::raw("Restore? ["),
        ratatui::text::Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("]es / ["),
        ratatui::text::Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("]o"),
    ]));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));

    let popup_area = centered_rect(70, 30, area);

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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

