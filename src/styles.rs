use tui::style::{Color, Modifier, Style};

pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub highlight: Color,
    pub negative_text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(42, 49, 56),
            text: Color::White,
            secondary: Color::Yellow,
            tertiary: Color::Green,
            highlight: Color::LightBlue,
            negative_text: Color::Black,
        }
    }
}

impl Theme {
    pub fn highlighted_snippet_style(&self) -> Style {
        Style::default().bg(self.highlight).fg(self.negative_text)
    }

    pub fn unhighlighted_snippet_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn cursor_style(&self) -> Style {
        Style::default().bg(self.secondary).fg(self.negative_text)
    }

    pub fn highlighted_title_style(&self) -> Style {
        Style::default()
            .bg(self.secondary)
            .fg(self.negative_text)
            .add_modifier(Modifier::UNDERLINED)
    }

    pub fn unhighlighted_title_style(&self) -> Style {
        Style::default()
            .fg(self.tertiary)
            .add_modifier(Modifier::UNDERLINED)
    }

    pub fn window_background(&self) -> Style {
        Style::default().bg(self.background)
    }

    pub fn selected_option(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(Modifier::UNDERLINED)
    }

    pub fn unselected_option(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn loading(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn block_border_unfocus(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn block_border_focus(&self) -> Style {
        Style::default().fg(self.secondary)
    }
}
