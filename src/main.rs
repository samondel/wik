mod app;
mod caching;
mod parsing;
mod styles;
mod ui;
mod utils;
mod widgets;
mod wikipedia;

use crate::app::App;
use app::{ActionMenu, AppState, CursorDirection, ScrollDirection, TypeableState};
use caching::CachingSession;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{size, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use htmd::HtmlToMarkdown;
use std::{
    error::Error,
    io::{Read, Write},
    time::{Duration, Instant},
};
use std::{fs::File, io};
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::{Terminal, TerminalOptions, Viewport};
use dialoguer::Input;

const APP_REFRESH_TIME_MILLIS: u64 = 16;
const APP_DEFAULT_MARGIN: u16 = 2;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    let mut fixed_size = false;
    let mut size = size()?;
    let mut margin:u16 = APP_DEFAULT_MARGIN;
    let mut ascii = false;
    if size.0 < 1 || size.1 < 1 {
        fixed_size = true;
        size = prompt_for_size()?;
        margin = get_dimension("margin size");
        ascii = get_bool("ASCII");
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let area = Rect::new(0, 0, size.0, size.1);
    let mut terminal = match fixed_size {
        true => Terminal::with_options(backend, TerminalOptions{viewport: Viewport::fixed(area)})?,
        false => Terminal::new(backend)?,
    };
    let mut app = App::new();
    app.is_running = true;
    app.ascii = ascii;

    // Main loop
    loop {
        if !app.is_running {
            break;
        }
        terminal.draw(|f| ui::draw(f, &app, margin))?;

        if event::poll(Duration::from_millis(APP_REFRESH_TIME_MILLIS))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Title => match key.code {
                        KeyCode::Enter => {
                            app.state = AppState::Search;
                        }
                        _ => {}
                    },
                    AppState::Search => match key.code {
                        KeyCode::Esc => {
                            // Enter Escape menu, from where one can exit normally
                            app.state = AppState::SearchMenu;
                        }
                        KeyCode::F(1) => {
                            // Just-in-case exit
                            app.is_running = false
                        }
                        KeyCode::Char(c) => {
                            // Append character to input
                            app.search.type_char(c);
                        }
                        KeyCode::Backspace => {
                            app.search.backspace();
                        }
                        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.search.move_cursor_to_start();
                        }
                        KeyCode::Left => {
                            app.search.move_cursor_one_step(CursorDirection::LEFT);
                        }
                        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.search.move_cursor_to_end();
                        }
                        KeyCode::Right => {
                            app.search.move_cursor_one_step(CursorDirection::RIGHT);
                        }
                        KeyCode::Enter => {
                            if app.search.text_box_is_highlighted {
                                app.load_wikipedia_search_query();
                                app.search.text_box_is_highlighted = false;
                            } else {
                                app.view_selected_article();
                            }
                        }
                        KeyCode::F(2) => {
                            app.view_selected_article();
                        }
                        KeyCode::Up => {
                            app.search.scroll_results(ScrollDirection::UP);
                        }
                        KeyCode::Down => {
                            app.search.scroll_results(ScrollDirection::DOWN);
                        }

                        _ => {}
                    },
                    AppState::SearchMenu => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Search;
                        }

                        KeyCode::Up => {
                            app.search_menu.scroll(ScrollDirection::UP);
                        }

                        KeyCode::Down => {
                            app.search_menu.scroll(ScrollDirection::DOWN);
                        }

                        KeyCode::Enter => {
                            app.search_menu.get_selected_action()(&mut app);
                        }

                        KeyCode::F(1) => {
                            // Just-in-case exit
                            app.is_running = false;
                        }
                        _ => {}
                    },
                    AppState::Credit => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::SearchMenu;
                        }

                        KeyCode::Up => {
                            app.credit.scroll(ScrollDirection::UP);
                        }
                        KeyCode::Down => {
                            app.credit.scroll(ScrollDirection::DOWN);
                        }

                        KeyCode::Enter => {
                            app.credit.get_selected_action()(&mut app);
                        }

                        _ => {}
                    },
                    AppState::Article => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::ArticleMenu;
                        }
                        _ => {}
                    },
                    AppState::ArticleMenu => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Article;
                        }
                        KeyCode::Up => {
                            app.article_menu.scroll(ScrollDirection::UP);
                        }

                        KeyCode::Down => {
                            app.article_menu.scroll(ScrollDirection::DOWN);
                        }

                        KeyCode::Enter => {
                            app.article_menu.get_selected_action()(&mut app);
                        }
                        _ => {}
                    },
                    _ => app.is_running = false,
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    CachingSession::clear_caches()?;

    /*
    println!("Getting your page...");
    let start_time = Instant::now();

    let url = "https://en.wikipedia.org/w/rest.php/v1/page/First 100 days of the first Donald Trump presidency/html";

    // "https://en.wikipedia.org/w/api.php?action=parse&page=Presidency_of_Donald_Trump&prop=text&format=json";
    // let response = reqwest::blocking::get(url)?.json::<WikiPageResponse>()?;
    let response = reqwest::blocking::get(url)?;
    let html_content = response.text()?;
    println!(
        "Download done in {} seconds",
        start_time.elapsed().as_millis()
    );

    // Print the extracted text content
    // println!("Title: {}", response.parse.title);
    // println!("Page ID: {}", response.parse.pageid);

    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "table", "sup"])
        .build();

    match converter.convert(&html_content) {
        Ok(content) => {
            // println!("Content:\n{}", content);
            let mut file = File::create("./earth.md")?;
            file.write_all(content.as_bytes())?;
        }
        Err(e) => {
            println!("uhoh!!!\n{}", e);
        }
    }
    println!(
        "Conversion done in {} seconds",
        start_time.elapsed().as_millis()
    );

    let mut markdown_text: String = String::new();
    File::open("./earth.md")?.read_to_string(&mut markdown_text)?;
    let mut spans = parsing::parse_markdown(&markdown_text);

    println!(
        "Parsing done in {} seconds",
        start_time.elapsed().as_millis()
    );

    spans = wikipedia::remove_unnecessary_spans(spans);

    for span in spans.iter() {
        println!("{}", span);
    }
    */

    Ok(())
}

fn prompt_for_size() -> Result<(u16, u16), std::io::Error> {
    eprintln!("Unable to automatically determine console dimensions.");
    let width = get_dimension("columns");
    let height = get_dimension("rows");
    return Ok((width, height))
}

fn get_dimension(dimension_name:&str) -> u16 {
    loop {
        let input: String = Input::new()
            .with_prompt(format!("Enter {}", dimension_name))
            .interact_text()
            .unwrap();
        match input.as_str().parse::<u16>() {
            Ok(dimension) => return dimension,
            Err(_e) => {
                eprintln!("Invalid input, please enter a positive integer.");
                continue;
            },
        };
    };
}

fn get_bool(bool_name:&str) -> bool {
    loop {
        let mut input: String = Input::new()
            .with_prompt(bool_name)
            .interact_text()
            .unwrap();
        input.make_ascii_lowercase();
        match input.as_str() {
                "y" => return true,
                "yes" => return true,
                "n" => return false,
                "no" => return false,
                _ => {
                    eprintln!("Enter Y or N.");
                    continue;
                },
            };
    };
}

