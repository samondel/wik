use std::sync::{MutexGuard, TryLockError, TryLockResult};

use crate::app::App;
use crate::styles::{cursor_style, highlighted_title_style, unhighlighted_title_style};
use tui::widgets::Wrap;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::wikipedia::SearchResult;

use substring::Substring;

pub fn draw<'a, B: Backend>(frame: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(frame.size());

    // Search input box
    let mut input_text = app.input.to_owned();
    input_text.push(' ');
    let pre_highlight = input_text.substring(0, app.cursor_pos);
    let highlight_char = input_text.substring(app.cursor_pos, app.cursor_pos + 1);
    let post_highlight = input_text.substring(app.cursor_pos + 1, input_text.len());
    let input = Paragraph::new(vec![Spans::from(vec![
        Span::raw(pre_highlight),
        Span::styled(highlight_char, cursor_style()),
        Span::raw(post_highlight),
    ])])
    .style(Style::default().fg(Color::Yellow))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search Wikipedia"),
    );

    frame.render_widget(input, chunks[0]);
    let mut is_loading = false;
    match app.is_loading_query.try_lock() {
        Ok(is_loading_guard) => {
            if *is_loading_guard {
                is_loading = true;
            }
        }
        Err(_) => {}
    }

    let mut available_results: TryLockResult<MutexGuard<'_, Vec<SearchResult>>> =
        Err(TryLockError::WouldBlock);

    if !is_loading {
        available_results = app.results.try_lock();
    } else {
        frame.render_widget(
            Paragraph::new("Loading...")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Results")),
            chunks[1],
        );
    }

    match available_results {
        Ok(result_guard) => {
            // Collect spans into a Vec<Spans>
            let results = result_guard;
            let spans: Vec<Spans> = results
                .iter()
                .enumerate()
                .flat_map(|(index, search_result)| -> Vec<Spans<'_>> {
                    let selected_index: usize = 0; // Update this logic as per your selection mechanism
                    let title_style = if index == selected_index {
                        highlighted_title_style()
                    } else {
                        unhighlighted_title_style()
                    };
                    let title_span = Span::styled(search_result.title.clone(), title_style);
                    if index == selected_index {
                        // let snippet_span = Span::styled(search_result.snippet.clone(), unhighlighted_snippet_style());
                        // let snippet_spans = search_result.highlighted_snippets();

                        vec![
                            Spans::from(vec![title_span]),
                            // snippet_spans,
                            SearchResult::highlighted_snippets(&search_result),
                            Spans::from(vec![Span::raw("")]),
                        ]
                    } else {
                        vec![Spans::from(vec![title_span])]
                    }
                })
                .collect(); // Collect spans into a Vec<Spans>

            // Create the Paragraph widget
            frame.render_widget(
                Paragraph::new(spans)
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .wrap(Wrap { trim: true }),
                chunks[1],
            );
        }
        Err(_) => {}
    }
}
