const GLYPH_TICK_CURSOR: &str = "▼";
const GLYPH_TICK_MAJOR: &str = "┬";
const GLYPH_TICK_LINE: &str = "─";
const GLYPH_BAR_PLAYED: &str = "⠶";
const GLYPH_BAR_UNPLAYED: &str = "⠤";
const GLYPH_MARKER_KEYFRAME: &str = "^";
const GLYPH_MARKER_START: &str = "[";
const GLYPH_MARKER_END: &str = "]";
const GLYPH_MARKER_START_SEL: &str = "▶";
const GLYPH_MARKER_END_SEL: &str = "◀";

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

fn format_time_ruler(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "00:00".to_string();
    }
    let minutes = (seconds / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    let millis = ((seconds % 1.0) * 10.0).floor() as u64; // tenths of a second
    if seconds < 60.0 {
        format!("{}.{}", secs, millis)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

pub fn render(f: &mut Frame, state: &mut AppState, area: Rect, theme: &Theme) {
    let duration = state.timeline_state.duration;

    if duration <= 0.0 || area.height < 5 || area.width == 0 {
        return;
    }

    // Fill the pane area with the timeline background and horizontal padding
    let block = Block::default()
        .style(theme.timeline_bg())
        .padding(Padding::new(2, 2, 0, 0));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Time ruler labels
            Constraint::Length(1), // Separator line with ticks & top cursor guide
            Constraint::Length(2), // Thicker Progress / Segment bar (2 rows high)
            Constraint::Length(1), // Keyframe / Cursor marker row
        ])
        .split(inner_area);

    let ruler_area = chunks[0];
    let sep_area = chunks[1];
    let bar_area = chunks[2];
    let marker_area = chunks[3];

    // Save actual timeline render rect coordinates in AppState for mouse hit-testing
    // This spans the 2-row progress bar and the 1-row marker row
    state.timeline_rect = (bar_area.x, bar_area.y, bar_area.width, 3);

    let width = bar_area.width as usize;

    let (start_time, end_time) = state.timeline_state.visible_range(width as u16);
    let visible_duration = end_time - start_time;

    // We want roughly 6 intervals across the screen
    let approx_interval = visible_duration / 6.0;
    
    // Nearest step
    let steps = [
        0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 
        60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0
    ];
    let s = steps.iter()
        .min_by(|&&a: &&f64, &&b: &&f64| {
            let diff_a = (a - approx_interval).abs();
            let diff_b = (b - approx_interval).abs();
            diff_a.partial_cmp(&diff_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap_or(10.0);

    let mut ruler_chars = vec![' '; width];
    let mut tick_chars = vec![GLYPH_TICK_LINE.chars().next().unwrap(); width];

    let first_tick_time = (start_time / s).floor() * s;
    let mut tick_time = first_tick_time;
    while tick_time <= end_time {
        if tick_time >= start_time {
            let px = state.timeline_state.time_to_pixel(tick_time, width as u16) as i32;
            if px >= 0 && px < width as i32 {
                // Place a tick mark
                tick_chars[px as usize] = GLYPH_TICK_MAJOR.chars().next().unwrap();
                
                // Format the label
                let label = format_time_ruler(tick_time);
                let label_len = label.chars().count();
                // Center the label above the tick
                let label_start = (px - (label_len as i32 / 2)).max(0) as usize;
                
                // Write the label to ruler_chars, making sure it doesn't overflow the width
                for (offset, ch) in label.chars().enumerate() {
                    let write_pos = label_start + offset;
                    if write_pos < width {
                        ruler_chars[write_pos] = ch;
                    }
                }
            }
        }
        tick_time += s;
    }

    let cursor_px = state.timeline_state.time_to_pixel(state.current_time, width as u16) as i32;

    // Render separator line with top cursor guide
    if cursor_px >= 0 && cursor_px < width as i32 {
        tick_chars[cursor_px as usize] = GLYPH_TICK_CURSOR.chars().next().unwrap();
    }


    // Convert ruler chars to line
    let ruler_str: String = ruler_chars.into_iter().collect();
    let ruler_line = Line::from(Span::styled(ruler_str, Style::default().fg(theme.muted)));
    f.render_widget(Paragraph::new(ruler_line).block(Block::default().style(theme.timeline_bg())), ruler_area);

    // Convert tick chars to line
    // Convert tick chars to line with run-length encoding
    let mut tick_spans = Vec::new();
    if width > 0 {
        let mut start = 0;
        while start < width {
            let ch = tick_chars[start];
            let mut end = start + 1;
            while end < width && tick_chars[end] == ch {
                end += 1;
            }

            let style = if ch == GLYPH_TICK_CURSOR.chars().next().unwrap() {
                Style::default().fg(theme.heading).add_modifier(Modifier::BOLD)
            } else if ch == GLYPH_TICK_MAJOR.chars().next().unwrap() {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.muted)
            };

            let s: String = tick_chars[start..end].iter().collect();
            tick_spans.push(Span::styled(s, style));
            start = end;
        }
    }

    f.render_widget(Paragraph::new(Line::from(tick_spans)).block(Block::default().style(theme.timeline_bg())), sep_area);

    // 1. Thicker Progress Bar (Solid blocks)
    let played_color = theme.accent;
    let unplayed_color = Color::Rgb(50, 50, 50);
    
    // Fill initially with unplayed dark braille dots (⠤)
    let mut bar_spans_top = vec![Span::styled(GLYPH_BAR_UNPLAYED, Style::default().fg(unplayed_color)); width];
    let mut bar_spans_bottom = vec![Span::styled(GLYPH_BAR_UNPLAYED, Style::default().fg(unplayed_color)); width];
    // Fill played portion (up to the cursor) with thick braille dots (⠶)
    let cursor_clamp = cursor_px.clamp(0, width as i32) as usize;
    for x in 0..cursor_clamp {
        bar_spans_top[x] = Span::styled(GLYPH_BAR_PLAYED, Style::default().fg(played_color));
        bar_spans_bottom[x] = Span::styled(GLYPH_BAR_PLAYED, Style::default().fg(played_color));
    }

    // Segment color palette
    let palette = theme.segment_palette();

    // Fill segments on progress bar
    for (i, segment) in state.segments.iter().enumerate() {
        let is_selected = Some(i) == state.selected_segment;
        let color = palette[i % palette.len()];
        let start_px = state.timeline_state.time_to_pixel(segment.start_seconds, width as u16) as i32;
        let end_px = state.timeline_state.time_to_pixel(segment.end_seconds, width as u16) as i32;
        let start_clamp = start_px.clamp(0, width as i32) as usize;
        let end_clamp = end_px.clamp(0, width as i32) as usize;

        for x in start_clamp..end_clamp {
            let glyph = if x < cursor_clamp { GLYPH_BAR_PLAYED } else { GLYPH_BAR_UNPLAYED };
            bar_spans_top[x] = Span::styled(glyph, Style::default().bg(color).fg(theme.highlight_fg));
            bar_spans_bottom[x] = Span::styled(glyph, Style::default().bg(color).fg(theme.highlight_fg));
        }

        // Add inward arrows for selected segment corners on the bar
        if is_selected {
            if start_clamp < width {
                bar_spans_top[start_clamp] = Span::styled(GLYPH_MARKER_START_SEL, Style::default().bg(color).fg(theme.highlight_fg).add_modifier(Modifier::BOLD));
                bar_spans_bottom[start_clamp] = Span::styled(GLYPH_MARKER_START_SEL, Style::default().bg(color).fg(theme.highlight_fg).add_modifier(Modifier::BOLD));
            }
            if end_clamp > 0 && end_clamp <= width {
                bar_spans_top[end_clamp - 1] = Span::styled(GLYPH_MARKER_END_SEL, Style::default().bg(color).fg(theme.highlight_fg).add_modifier(Modifier::BOLD));
                bar_spans_bottom[end_clamp - 1] = Span::styled(GLYPH_MARKER_END_SEL, Style::default().bg(color).fg(theme.highlight_fg).add_modifier(Modifier::BOLD));
            }
        }

        // Overlay segment label inside the top row
        let label = segment.label.as_deref().unwrap_or("");
        if !label.is_empty() {
            let label_len = label.chars().count();
            let seg_width = end_clamp.saturating_sub(start_clamp);
            if seg_width >= label_len + 2 {
                let offset = (seg_width - label_len) / 2;
                for (ch_idx, ch) in label.chars().enumerate() {
                    let x = start_clamp + offset + ch_idx;
                    if x < width {
                        bar_spans_top[x] = Span::styled(
                            ch.to_string(),
                            Style::default().bg(color).fg(theme.highlight_fg).add_modifier(Modifier::BOLD)
                        );
                    }
                }
            }
        }
    }

    // Render the progress bar across two vertical rows
    let bar_paragraph = Paragraph::new(vec![Line::from(bar_spans_top), Line::from(bar_spans_bottom)])
        .block(Block::default().style(theme.timeline_bg()));
    f.render_widget(bar_paragraph, bar_area);

    // 2. Keyframe and Marker Row
    let mut marker_spans = vec![Span::raw(" "); width];

    // Draw keyframes as '^'
    if let Some(metadata) = &state.metadata {
        let (start_time, end_time) = state.timeline_state.visible_range(width as u16);
        let lo = metadata.keyframes_seconds.partition_point(|&k| k < start_time);
        let hi = metadata.keyframes_seconds.partition_point(|&k| k <= end_time);
        for &kf in &metadata.keyframes_seconds[lo..hi] {
            let px = state.timeline_state.time_to_pixel(kf, width as u16) as i32;
            if px >= 0 && px < width as i32 {
                marker_spans[px as usize] = Span::styled(GLYPH_MARKER_KEYFRAME, Style::default().fg(theme.muted));
            }
        }
    }

    // Draw segment boundary markers
    for (i, segment) in state.segments.iter().enumerate() {
        let is_selected = Some(i) == state.selected_segment;
        let color = palette[i % palette.len()];
        let start_px = state.timeline_state.time_to_pixel(segment.start_seconds, width as u16) as i32;
        let end_px = state.timeline_state.time_to_pixel(segment.end_seconds, width as u16) as i32;
        let (left_marker, right_marker) = if is_selected {
            (GLYPH_MARKER_START_SEL, GLYPH_MARKER_END_SEL)
        } else {
            (GLYPH_MARKER_START, GLYPH_MARKER_END)
        };
        if start_px >= 0 && start_px < width as i32 {
            marker_spans[start_px as usize] = Span::styled(left_marker, Style::default().fg(color).add_modifier(Modifier::BOLD));
        }
        if end_px >= 0 && end_px < width as i32 {
            marker_spans[end_px as usize] = Span::styled(right_marker, Style::default().fg(color).add_modifier(Modifier::BOLD));
        }
    }

    // Draw pending in-point as green '['
    if let Some(in_point) = state.pending_in_point {
        let px = state.timeline_state.time_to_pixel(in_point, width as u16) as i32;
        if px >= 0 && px < width as i32 {
            marker_spans[px as usize] = Span::styled("[", Style::default().fg(theme.success).add_modifier(Modifier::BOLD));
        }
    }

    // Draw cursor on marker row
    if cursor_px >= 0 && cursor_px < width as i32 {
        marker_spans[cursor_px as usize] = Span::styled("▲", Style::default().fg(theme.heading).add_modifier(Modifier::BOLD));
    }

    let marker_line = Line::from(marker_spans);
    f.render_widget(Paragraph::new(marker_line).block(Block::default().style(theme.timeline_bg())), marker_area);
}
