use ratatui::{Frame, layout::{Constraint, Direction, Layout}};
use crate::app::AppState;
use crate::model::AppMode;

pub mod header;
pub mod help;
pub mod segments;
pub mod status;
pub mod theme;
pub mod timeline;
pub mod player_info;

pub fn render(f: &mut Frame, state: &mut AppState) {
    let theme = theme::Theme::default();

    let area = if f.area().width >= 10 && f.area().height >= 10 {
        f.area().inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 2,
        })
    } else {
        f.area()
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),      // Header (text + bottom border)
            Constraint::Min(10),        // Main content area
            Constraint::Length(5),      // Timeline (1 line ruler + 1 line separator/ticks + 2 lines thick bar + 1 line markers)
            Constraint::Length(1),      // Status bar
        ])
        .split(area);

    header::render(f, state, chunks[0], &theme);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Player status & selection
            Constraint::Length(1),      // Vertical separator
            Constraint::Percentage(50), // Right: Segments list
        ])
        .split(chunks[1]);

    player_info::render(f, state, main_chunks[0], &theme);

    use ratatui::widgets::{Block, Borders};
    use ratatui::style::Style;
    let vertical_sep = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme.border));
    f.render_widget(vertical_sep, main_chunks[1]);

    segments::render(f, state, main_chunks[2], &theme);

    timeline::render(f, state, chunks[2], &theme);
    status::render(f, state, chunks[3], &theme);

    if state.mode == AppMode::Help {
        help::render(f, f.area(), &theme);
    }
    if state.mode == AppMode::EditLabel {
        render_edit_label_popup(f, state, f.area(), &theme);
    }
    if state.mode == AppMode::SessionRestore {
        render_restore_popup(f, state, f.area(), &theme);
    }
    if state.is_probing {
        render_probing_popup(f, state, f.area(), &theme);
    }
}

pub(crate) fn draw_popup_banner(f: &mut Frame, popup_area: ratatui::layout::Rect, title: &str, theme: &theme::Theme) {
    use ratatui::widgets::{Clear, Paragraph};
    use ratatui::text::{Line, Span};
    use ratatui::style::{Style, Modifier};
    let display_text = format!(" {} ", title.to_uppercase());
    let width = display_text.len() as u16;
    if popup_area.y == 0 {
        return;
    }
    let banner_area = ratatui::layout::Rect::new(
        popup_area.x + (popup_area.width.saturating_sub(width)) / 2,
        popup_area.y - 1,
        width.min(popup_area.width),
        1,
    );
    f.render_widget(Clear, banner_area);
    let p = Paragraph::new(Line::from(vec![Span::styled(
        display_text,
        Style::default()
            .fg(theme.highlight_fg)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(p, banner_area);
}

fn render_restore_popup(f: &mut Frame, state: &AppState, area: ratatui::layout::Rect, theme: &theme::Theme) {
    use ratatui::widgets::{Block, Clear, Paragraph};
    use ratatui::style::{Style, Modifier};
    use ratatui::layout::Alignment;

    let block = Block::default().style(theme.popup_bg());

    let mut text = Vec::new();
    text.push(ratatui::text::Line::from(""));

    if let Some(session) = &state.pending_session {
        let seg_count = session.segments.len();
        text.push(ratatui::text::Line::from(vec![
            ratatui::text::Span::raw("A previous session with "),
            ratatui::text::Span::styled(format!("{}", seg_count), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            ratatui::text::Span::raw(" segment(s) was found."),
        ]));

        if let Some(source_path) = &state.source_path {
            let validation = crate::session::validate(session, source_path);
            if !validation.modified_match {
                text.push(ratatui::text::Line::from(""));
                text.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    "⚠️ Warning: Source file modified time does not match session save time.",
                    Style::default().fg(theme.destructive).add_modifier(Modifier::BOLD)
                )));
                text.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    "Keyframe positions could differ.",
                    Style::default().fg(theme.destructive)
                )));
            }
        }
    } else {
        text.push(ratatui::text::Line::from("A previous session was found."));
    }

    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from(vec![
        ratatui::text::Span::raw("Restore? ["),
        ratatui::text::Span::styled("Y", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("]es / ["),
        ratatui::text::Span::styled("N", Style::default().fg(theme.destructive).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("]o"),
    ]));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.fg));

    let popup_area = centered_rect(70, 30, area);

    f.render_widget(Clear, popup_area);
    draw_popup_banner(f, popup_area, "RESTORE SESSION?", theme);
    f.render_widget(paragraph, popup_area);
}

fn render_edit_label_popup(f: &mut Frame, state: &AppState, area: ratatui::layout::Rect, theme: &theme::Theme) {
    use ratatui::widgets::{Block, Clear, Paragraph};
    use ratatui::style::{Style, Modifier};
    use ratatui::layout::Alignment;

    let block = Block::default()
        .style(theme.popup_bg());

    let mut text = Vec::new();
    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from("Enter label for segment:"));
    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from(ratatui::text::Span::styled(
        format!(" > {}_ ", state.label_input),
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    )));
    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from(vec![
        ratatui::text::Span::raw("["),
        ratatui::text::Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("] Save  ·  ["),
        ratatui::text::Span::styled("Esc", Style::default().fg(theme.destructive).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw("] Cancel"),
    ]));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.fg));

    let popup_area = centered_rect(60, 25, area);

    f.render_widget(Clear, popup_area);
    draw_popup_banner(f, popup_area, "EDIT LABEL", theme);
    f.render_widget(paragraph, popup_area);
}

fn render_probing_popup(f: &mut Frame, _state: &AppState, area: ratatui::layout::Rect, theme: &theme::Theme) {
    use ratatui::widgets::{Block, Clear, Paragraph};
    use ratatui::style::Style;
    use ratatui::layout::Alignment;

    let block = Block::default()
        .style(theme.popup_bg());

    let mut text = Vec::new();
    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::from("Probing video metadata..."));
    text.push(ratatui::text::Line::from("Please wait."));
    text.push(ratatui::text::Line::from(""));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.fg));

    let popup_area = centered_rect(50, 20, area);

    f.render_widget(Clear, popup_area);
    draw_popup_banner(f, popup_area, "LOADING", theme);
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
