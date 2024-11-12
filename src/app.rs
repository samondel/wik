use crate::wikipedia::SearchResult;
use crate::{caching::CachingSession, utils::Shared};

use std::sync::{Arc, Mutex};

pub struct App {
    pub input: String,
    pub results: Shared<Vec<SearchResult>>,
    pub cursor_pos: usize,
    pub is_loading_query: Shared<bool>,
    pub cache: Shared<CachingSession>,
}

impl Default for App {
    fn default() -> Self {
        App {
            input: String::new(),
            results: Arc::new(Mutex::new(Vec::new())),
            cursor_pos: 0,
            is_loading_query: Arc::new(Mutex::new(false)),
            cache: Arc::new(Mutex::new(CachingSession::new())),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn is_this_lockable(&self) -> bool {
        match self.is_loading_query.try_lock() {
            Ok(is_loading) => !(*is_loading),
            Err(_) => false,
        }
    }

    pub fn scroll_results(&self, delta: i8) {
        if delta > 0 {
            (*self.results.lock().unwrap()).rotate_left(1);
        } else {
            (*self.results.lock().unwrap()).rotate_right(1);
        }
    }

    pub fn move_cursor_one_step(&mut self, delta: i8) {
        // Limit cursor_pos to be between 0 and (length of the input) - 1
        let delta: i8 = if delta < 0 { -1 } else { 1 };
        if delta < 0 {
            self.cursor_pos = self.cursor_pos.saturating_sub(1);
        } else {
            if self.cursor_pos < self.input.len() {
                self.cursor_pos += 1;
            }
        }
        self.cursor_pos = self.cursor_pos.clamp(0, self.input.len() + 1)
    }

    pub fn move_cursor_to_start(&mut self) -> () {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_to_end(&mut self) -> () {
        self.cursor_pos = self.input.len();
    }
}

// pub type ConcurrentApp = Arc<Mutex<App>>;
