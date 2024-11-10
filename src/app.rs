use crate::caching::CachingSession;
use crate::wikipedia::SearchResult;
use std::fmt;

use std::sync::{Arc, Mutex, MutexGuard};

pub struct App {
    pub input: String,
    pub results: Arc<Mutex<Vec<SearchResult>>>,
    pub cursor_pos: usize,
    pub is_loading_query: Arc<Mutex<bool>>,
    pub cache: Arc<Mutex<CachingSession>>,
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

    pub fn start_search(&self) {
        *self.is_loading_query.lock().unwrap() = true;
    }

    pub fn stop_search(&self) {
        *self.is_loading_query.lock().unwrap() = false;
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

/* impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let results = *self.results.lock().unwrap();
        write!(
            f,
            "App\n\tinput: {}\n\tresults: {}",
            self.input,
            self.results
                .lock()
                .unwrap()
                .iter()
                .map(|r| r.title.to_owned())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
 */
