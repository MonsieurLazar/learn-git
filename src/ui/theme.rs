use ratatui::style::{Color, Modifier, Style};

const BG: Color = Color::Rgb(16, 20, 24);
const PANEL: Color = Color::Rgb(22, 27, 33);
const PANEL_ALT: Color = Color::Rgb(28, 34, 41);
const BORDER: Color = Color::Rgb(88, 102, 116);
const TEXT: Color = Color::Rgb(226, 217, 202);
const MUTED: Color = Color::Rgb(150, 145, 136);
const ACCENT: Color = Color::Rgb(227, 144, 62);
const ACCENT_SOFT: Color = Color::Rgb(109, 166, 174);

pub fn app_background() -> Style {
    Style::default().bg(BG).fg(TEXT)
}

pub fn panel_background() -> Style {
    Style::default().bg(PANEL)
}

pub fn panel_alt_background() -> Style {
    Style::default().bg(PANEL_ALT)
}

pub fn muted_text() -> Style {
    Style::default().fg(MUTED)
}

pub fn accent_text() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn soft_accent_text() -> Style {
    Style::default().fg(ACCENT_SOFT)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}
