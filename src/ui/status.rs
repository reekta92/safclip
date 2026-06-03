use ratatui::{
    layout::Rect,
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;
pub fn render(f: &mut Frame, state: &AppState, area: Rect, theme: &Theme) {
    let content = if state.mode == crate::model::AppMode::SessionRestore {
        state.status_message.as_deref().unwrap_or("Session found. Restore? [Y/n]")
    } else if let Some(msg) = &state.status_message {
        msg.as_str()
    } else {
        "q:quit ?:help Space:play/pause ←→:seek a/d:mark Del:del e/E:export Tab:player u/Ctrl+R:undo/redo"
    };

    let status = Paragraph::new(content)
        .block(Block::default().style(theme.hint_line_bg()).padding(Padding::new(2, 2, 0, 0)))
        .style(ratatui::style::Style::default().fg(theme.muted));

    f.render_widget(status, area);
}
