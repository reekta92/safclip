use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let content = if state.mode == crate::model::AppMode::SessionRestore {
        state.status_message.as_deref().unwrap_or("Session found. Restore? [Y/n]")
    } else if let Some(msg) = &state.status_message {
        msg.as_str()
    } else {
        "q:quit ?:help Space:play/pause ←→:seek a/d:mark Del:del e/E:export Tab:player u/Ctrl+R:undo/redo"
    };

    let status = Paragraph::new(content)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(status, area);
}
