use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FormattedSpan {
    pub index: usize,
    pub text: String,
    pub is_heading: bool,
    pub heading_level: usize,
    pub link: Option<String>,
    pub is_break: bool,
}

impl Display for FormattedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_heading {
            write!(
                f,
                "index: {}, text: {}, {} heading",
                self.index, self.text, self.heading_level
            )
        } else if let Some(link_text) = &self.link {
            write!(
                f,
                "index: {}, text: {}, link: {}",
                self.index, self.text, link_text
            )
        } else if self.is_break {
            write!(f, "index: {}, line break", self.index)
        } else {
            write!(f, "index: {}, text: {}", self.index, self.text)
        }
    }
}

impl Default for FormattedSpan {
    fn default() -> Self {
        Self {
            index: 0,
            text: String::from(""),
            is_heading: false,
            heading_level: 0,
            link: None,
            is_break: false,
        }
    }
}
