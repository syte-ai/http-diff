use ratatui::style::Color;

pub enum ThemeType {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub background: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub highlighted: Color,
    pub gray: Color,
    pub white: Color,
    pub black: Color,
}

pub fn get_dark_theme() -> Theme {
    Theme {
        background: Color::Indexed(235),
        success: Color::Green,
        warning: Color::Yellow,
        error: Color::Red,
        highlighted: Color::White,
        gray: Color::Gray,
        white: Color::White,
        black: Color::Black,
    }
}

pub fn get_light_theme() -> Theme {
    Theme {
        background: Color::Indexed(019),
        success: Color::Indexed(076),
        warning: Color::Indexed(208),
        error: Color::Indexed(196),
        highlighted: Color::White,
        gray: Color::Gray,
        white: Color::White,
        black: Color::Black,
    }
}
