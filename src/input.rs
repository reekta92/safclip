use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};

#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    Quit,
    SeekForward(f64),
    SeekBackward(f64),
    SeekToStart,
    SeekToEnd,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    SetInPoint,
    SetOutPoint,
    ConfirmSegment,
    DeleteSegment,
    SelectPrevSegment,
    SelectNextSegment,
    SnapToKeyframe,
    SeekToSegmentStart,
    SeekToSegmentEnd,
    TogglePlay,
    Export,
    ExportMerged,
    ExportSelected,
    ToggleHelp,
    OpenFile(String),
    Undo,
    Redo,
    EditLabel,
    Cancel,
    SwitchPlayer,
    MousePress { button: MouseButton, row: u16, col: u16 },
    MouseDrag { row: u16, col: u16 },
    MouseRelease { row: u16, col: u16 },
    MouseScroll { up: bool, row: u16, col: u16 },
    RestoreSession,
    DiscardSession,
    None,
}

pub fn handle_event(event: Event) -> AppAction {
    match event {
        Event::Key(key) => handle_key_event(key),
        Event::Mouse(mouse) => handle_mouse_event(mouse),
        _ => AppAction::None,
    }
}

fn handle_key_event(key: KeyEvent) -> AppAction {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => AppAction::Quit,

        (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::NONE) => AppAction::SeekBackward(1.0),
        (KeyCode::Right, KeyModifiers::NONE) | (KeyCode::Char('l'), KeyModifiers::NONE) => AppAction::SeekForward(1.0),
        (KeyCode::Left, KeyModifiers::SHIFT) => AppAction::SeekBackward(5.0),
        (KeyCode::Right, KeyModifiers::SHIFT) => AppAction::SeekForward(5.0),
        (KeyCode::Left, KeyModifiers::ALT) => AppAction::SeekBackward(10.0),
        (KeyCode::Right, KeyModifiers::ALT) => AppAction::SeekForward(10.0),

        (KeyCode::Home, KeyModifiers::NONE) => AppAction::SeekToStart,
        (KeyCode::End, KeyModifiers::NONE) => AppAction::SeekToEnd,

        (KeyCode::Char(' '), KeyModifiers::NONE) => AppAction::TogglePlay,

        (KeyCode::Char('a'), KeyModifiers::NONE) => AppAction::SetInPoint,
        (KeyCode::Char('d'), KeyModifiers::NONE) => AppAction::SetOutPoint,

        (KeyCode::Enter, KeyModifiers::NONE) => AppAction::ConfirmSegment,
        (KeyCode::Delete, KeyModifiers::NONE) | (KeyCode::Char('x'), KeyModifiers::NONE) | (KeyCode::Char('s'), KeyModifiers::NONE) => AppAction::DeleteSegment,
        (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => AppAction::SelectPrevSegment,
        (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('j'), KeyModifiers::NONE) => AppAction::SelectNextSegment,
        (KeyCode::Char('H'), _) | (KeyCode::Char('h'), KeyModifiers::SHIFT) => AppAction::SeekToSegmentStart,
        (KeyCode::Char('L'), _) | (KeyCode::Char('l'), KeyModifiers::SHIFT) => AppAction::SeekToSegmentEnd,

        (KeyCode::Char('e'), KeyModifiers::NONE) => AppAction::Export,
        (KeyCode::Char('E'), _) | (KeyCode::Char('e'), KeyModifiers::SHIFT) => AppAction::ExportMerged,
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => AppAction::ExportSelected,

        (KeyCode::Char('K'), _) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => AppAction::SnapToKeyframe,

        (KeyCode::Char('+') | KeyCode::Char('='), _) => AppAction::ZoomIn,
        (KeyCode::Char('-'), KeyModifiers::NONE) => AppAction::ZoomOut,

        (KeyCode::Char('h'), KeyModifiers::ALT) => AppAction::PanLeft,
        (KeyCode::Char('l'), KeyModifiers::ALT) => AppAction::PanRight,

        (KeyCode::Char('u'), KeyModifiers::NONE) => AppAction::Undo,
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => AppAction::Redo,

        (KeyCode::Char('?'), _) | (KeyCode::Char('/'), KeyModifiers::SHIFT) => AppAction::ToggleHelp,

        (KeyCode::Esc, KeyModifiers::NONE) => AppAction::Cancel,

        (KeyCode::Char('y') | KeyCode::Char('Y'), KeyModifiers::NONE) => AppAction::RestoreSession,
        (KeyCode::Char('n') | KeyCode::Char('N'), KeyModifiers::NONE) => AppAction::DiscardSession,

        (KeyCode::Tab, KeyModifiers::NONE) => AppAction::SwitchPlayer,

        _ => AppAction::None,
    }
}

fn handle_mouse_event(mouse: MouseEvent) -> AppAction {
    match mouse.kind {
        MouseEventKind::Down(button) => AppAction::MousePress {
            button,
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::Drag(_) => AppAction::MouseDrag {
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::Up(_) => AppAction::MouseRelease {
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::ScrollUp => AppAction::MouseScroll {
            up: true,
            row: mouse.row,
            col: mouse.column,
        },
        MouseEventKind::ScrollDown => AppAction::MouseScroll {
            up: false,
            row: mouse.row,
            col: mouse.column,
        },
        _ => AppAction::None,
    }
}
