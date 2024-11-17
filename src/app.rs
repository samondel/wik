use crate::parsing::FormattedSpan;
use crate::styles::Theme;
use crate::utils::{create_shared, remainder, shared_copy};
use crate::wikipedia::{self, SearchResult, WikiSearchResponse};
use crate::{caching::CachingSession, utils::Shared};

use std::char;
use std::sync::{Arc, Mutex};

pub enum AppState {
    Title,
    Search,
    SearchMenu,
    Article,
    ArticleMenu,
    Credit,
}
pub type AppAction = Arc<dyn Fn(&mut App) + Send + Sync>;

pub struct ActionItem {
    label: String,
    action: AppAction,
}

impl ActionItem {
    pub fn new<F>(label: &str, action: F) -> Self
    where
        F: Fn(&mut App) + Send + Sync + 'static,
    {
        ActionItem {
            label: label.to_string(),
            action: Arc::new(action),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn action_clone(&self) -> AppAction {
        Arc::clone(&self.action)
    }
}

pub enum ScrollDirection {
    UP,
    DOWN,
}

pub enum CursorDirection {
    LEFT,
    RIGHT,
}

pub trait ActionMenu {
    fn total_options(&self) -> usize;
    fn set_index(&mut self, new_index: usize) -> ();
    fn get_index(&self) -> usize;
    fn get_options(&self) -> &Vec<ActionItem>;

    fn scroll(&mut self, scroll_direction: ScrollDirection) -> () {
        let total_options = self.total_options();

        match scroll_direction {
            ScrollDirection::DOWN => self.set_index(remainder(self.get_index() + 1, total_options)),
            ScrollDirection::UP => self.set_index(
                remainder(self.get_index() as i64 - 1, total_options as i64)
                    .try_into()
                    .unwrap_or(0),
            ),
        }
    }

    fn get_selected_action(&self) -> AppAction {
        let selected_option = &self.get_options()[self.get_index()];
        selected_option.action_clone()
    }
}

pub trait TypeableState {
    fn get_input(&self) -> String;
    fn insert_to_input_at_cursor(&mut self, c: char) -> ();
    fn remove_from_input_at_cursor(&mut self) -> ();
    fn get_cursor_pos(&self) -> usize;
    fn set_cursor_pos(&mut self, new_cursor_pos: usize) -> ();
    fn trigger_text_focus(&mut self) -> () {}

    fn move_cursor_to_start(&mut self) -> () {
        self.set_cursor_pos(0);
        self.trigger_text_focus();
    }

    fn move_cursor_to_end(&mut self) -> () {
        self.set_cursor_pos(self.get_input().len());
        self.trigger_text_focus();
    }

    fn move_cursor_one_step(&mut self, cursor_direction: CursorDirection) {
        // Limit cursor_pos to be between 0 and (length of the input) - 1
        match cursor_direction {
            CursorDirection::LEFT => {
                self.set_cursor_pos(self.get_cursor_pos().saturating_sub(1));
            }
            CursorDirection::RIGHT => {
                if self.get_cursor_pos() < self.get_input().len() {
                    self.set_cursor_pos(self.get_cursor_pos() + 1);
                }
            }
        }
        self.set_cursor_pos(self.get_cursor_pos().clamp(0, self.get_input().len() + 1));
        self.trigger_text_focus();
    }

    fn type_char(&mut self, c: char) {
        if !(self.get_cursor_pos() > self.get_input().len()) {
            self.insert_to_input_at_cursor(c);
            self.move_cursor_one_step(CursorDirection::RIGHT);
        }
        self.trigger_text_focus();
    }

    fn backspace(&mut self) {
        if !(self.get_input().is_empty()) {
            if self.get_cursor_pos() > 0 {
                self.remove_from_input_at_cursor();
                self.move_cursor_one_step(CursorDirection::LEFT);
            }
        }
        self.trigger_text_focus();
    }
}

pub struct TitleState {
    pub input: String,
    pub cursor_pos: usize,
}

impl TypeableState for TitleState {
    fn get_input(&self) -> String {
        self.input.clone()
    }

    fn insert_to_input_at_cursor(&mut self, c: char) -> () {
        self.input.insert(self.cursor_pos, c);
    }

    fn remove_from_input_at_cursor(&mut self) -> () {
        if self.cursor_pos <= self.input.len() && self.cursor_pos > 0 {
            self.input.remove(self.cursor_pos - 1);
        }
    }

    fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    fn set_cursor_pos(&mut self, new_cursor_pos: usize) -> () {
        self.cursor_pos = new_cursor_pos;
    }
}

pub struct SearchState {
    pub input: String,
    pub current_query: String,
    pub results: Shared<Vec<SearchResult>>,
    pub cursor_pos: usize,
    pub is_loading_query: Shared<bool>,
    pub selected_index: usize,
    pub text_box_is_highlighted: bool,
}

impl SearchState {
    pub fn currently_loading(&self) -> bool {
        match self.is_loading_query.try_lock() {
            Ok(is_loading) => (*is_loading),
            Err(_) => true,
        }
    }

    pub fn scroll_results(&mut self, scroll_direction: ScrollDirection) {
        if !self.currently_loading() {
            let results = self.results.lock().unwrap();
            if results.len() > 0 {
                match scroll_direction {
                    ScrollDirection::DOWN => {
                        self.selected_index = remainder(self.selected_index + 1, results.len());
                    }
                    ScrollDirection::UP => {
                        self.selected_index =
                            remainder(self.selected_index as i64 - 1, results.len() as i64)
                                as usize;
                    }
                }
            }
        }
        self.text_box_is_highlighted = false;
    }

    pub fn selected_search_result_title(&self) -> Option<String> {
        match self.results.lock().unwrap().get(self.selected_index) {
            Some(result) => Some(result.title.clone()),
            None => None,
        }
    }
}

impl TypeableState for SearchState {
    fn get_input(&self) -> String {
        self.input.clone()
    }

    fn insert_to_input_at_cursor(&mut self, c: char) -> () {
        self.input.insert(self.cursor_pos, c);
    }

    fn remove_from_input_at_cursor(&mut self) -> () {
        if self.cursor_pos <= self.input.len() && self.cursor_pos > 0 {
            self.input.remove(self.cursor_pos - 1);
        }
    }

    fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    fn set_cursor_pos(&mut self, new_cursor_pos: usize) -> () {
        self.cursor_pos = new_cursor_pos;
    }

    fn trigger_text_focus(&mut self) -> () {
        self.text_box_is_highlighted = true;
    }
}
pub struct MenuState {
    pub selected_index: usize,
    pub options: Vec<ActionItem>,
}

impl ActionMenu for MenuState {
    fn total_options(&self) -> usize {
        self.options.len()
    }

    fn set_index(&mut self, new_index: usize) -> () {
        self.selected_index = new_index;
    }

    fn get_index(&self) -> usize {
        self.selected_index
    }

    fn get_options(&self) -> &Vec<ActionItem> {
        &self.options
    }
}

pub struct CreditState {
    pub selected_index: usize,
    pub options: Vec<ActionItem>,
}

impl ActionMenu for CreditState {
    fn total_options(&self) -> usize {
        self.options.len()
    }

    fn set_index(&mut self, new_index: usize) -> () {
        self.selected_index = new_index;
    }

    fn get_index(&self) -> usize {
        self.selected_index
    }

    fn get_options(&self) -> &Vec<ActionItem> {
        &self.options
    }
}

pub struct ArticleState {
    pub article_name: String,
    pub markdown_spans: Shared<Vec<FormattedSpan>>,
    pub is_loading_article: Shared<bool>,
}

pub struct App {
    pub title: TitleState,
    pub search: SearchState,
    pub search_menu: MenuState,
    pub credit: CreditState,
    pub article: ArticleState,
    pub article_menu: MenuState,
    pub cache: Shared<CachingSession>,
    pub is_running: bool,
    pub state: AppState,
    pub theme: Theme,
    pub ascii: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            title: TitleState {
                input: String::new(),
                cursor_pos: 0,
            },
            search: SearchState {
                input: String::new(),
                current_query: String::new(),
                results: create_shared(Vec::new()),
                cursor_pos: 0,
                is_loading_query: create_shared(false),
                selected_index: 0,
                text_box_is_highlighted: true,
            },
            search_menu: MenuState {
                selected_index: 0,
                options: vec![],
            },
            credit: CreditState {
                selected_index: 0,
                options: vec![],
            },
            article: ArticleState {
                article_name: String::from("Philosophy"),
                markdown_spans: create_shared(Vec::new()),
                is_loading_article: create_shared(false),
            },
            article_menu: MenuState {
                selected_index: 0,
                options: vec![],
            },
            cache: create_shared(CachingSession::new()),
            is_running: false,
            state: AppState::Title,
            theme: Theme::default(),
            ascii: false,
        };

        app.search_menu.options = vec![
            ActionItem::new("Back", |app| app.state = AppState::Search),
            ActionItem::new("Credits", |app| app.state = AppState::Credit),
            ActionItem::new("Quit", |app| app.is_running = false),
        ];

        app.article_menu.options = vec![
            ActionItem::new("Back", |app| app.state = AppState::Article),
            ActionItem::new("Search", |app| app.state = AppState::Search),
            ActionItem::new("Quit", |app| app.is_running = false),
        ];

        app.credit.options = vec![
            ActionItem::new("Go to repo!", |_| {
                webbrowser::open("https://github.com/itsjustmustafa/wik").unwrap_or(())
            }),
            ActionItem::new("Back to menu", |app| app.state = AppState::SearchMenu),
        ];

        app
    }
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn load_wikipedia_search_query(&mut self) {
        if self.search.input.len() > 0 {
            if !self.search.currently_loading() {
                let input = self.search.input.clone();
                self.search.current_query = input.clone();

                let loading_flag = shared_copy(&self.search.is_loading_query);
                let app_results = shared_copy(&self.search.results);
                let caching_session = shared_copy(&self.cache);

                wikipedia::load_search_query_to_app(
                    input,
                    loading_flag,
                    app_results,
                    caching_session,
                );
            }
        }
    }

    pub fn view_selected_article(&mut self) {
        if let Some(title) = self.search.selected_search_result_title() {
            self.state = AppState::Article;
            self.article.article_name = title.clone();

            let markdown_spans = shared_copy(&self.article.markdown_spans);
            let loading_flag = shared_copy(&self.article.is_loading_article);
            let caching_session = shared_copy(&self.cache);

            wikipedia::load_article_to_app(
                title.clone(),
                loading_flag,
                markdown_spans,
                caching_session,
            );
        } else {
            self.state = AppState::SearchMenu;
        }
    }
}
