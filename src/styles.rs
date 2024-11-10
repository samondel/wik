use tui::style::{Color, Style};

pub fn highlighted_snippet_style() -> Style {
    Style::default().bg(Color::LightBlue).fg(Color::Black)
}

pub fn unhighlighted_snippet_style() -> Style {
    Style::default()
}

pub fn cursor_style() -> Style {
    Style::default().bg(Color::Yellow).fg(Color::Black)
}

pub fn highlighted_title_style() -> Style {
    Style::default().fg(Color::Black).bg(Color::Yellow)
}

pub fn unhighlighted_title_style() -> Style {
    Style::default().fg(Color::Green)
}
