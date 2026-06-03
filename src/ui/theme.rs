use ratatui::style::{Color, Style};

pub struct Theme {
    pub accent: Color,       // Cyan
    pub heading: Color,      // Yellow
    pub success: Color,      // Green
    pub destructive: Color,  // Red
    pub muted: Color,        // DarkGray
    pub text: Color,         // Reset
    pub fg: Color,           // White
    pub border: Color,       // DarkGray (for rare borders like popups)
    pub highlight_fg: Color, // Black
    pub highlight_bg: Color, // Cyan
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            heading: Color::Yellow,
            success: Color::Green,
            destructive: Color::Red,
            muted: Color::DarkGray,
            text: Color::Reset,
            fg: Color::White,
            border: Color::DarkGray,
            highlight_fg: Color::Black,
            highlight_bg: Color::Cyan,
        }
    }
}

impl Theme {
    pub fn header_bg(&self) -> Style {
        Style::default()
    }

    pub fn player_info_bg(&self) -> Style {
        Style::default()
    }

    pub fn segments_bg(&self) -> Style {
        Style::default()
    }

    pub fn timeline_bg(&self) -> Style {
        Style::default()
    }

    pub fn hint_line_bg(&self) -> Style {
        Style::default()
    }

    pub fn title_bar_bg(&self) -> Style {
        Style::default()
    }

    pub fn popup_bg(&self) -> Style {
        Style::default()
    }
}
