use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::Arc, thread};
use tui::text::{Span, Spans};

use crate::{
    app::App,
    caching::CachingSession,
    styles::{highlighted_snippet_style, unhighlighted_snippet_style},
    utils::Shared,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
}

const OPENING_TAG: &str = "<span class=\"searchmatch\">";
const CLOSING_TAG: &str = "</span>";

impl SearchResult {
    pub fn highlighted_snippets(search_results: &SearchResult) -> Spans {
        // The search results from the Wikipedia API encloses the parts of the snippet
        // that matches the search term with the values in OPENING_TAG and then CLOSING_TAG,
        // This provides the snippet with the matches highlighted.

        let mut spans = Vec::new();
        let parts: Vec<&str> = search_results.snippet.split(OPENING_TAG).collect();

        for (i, &part) in parts.iter().enumerate() {
            // the strings in `part` will start with the highlighted text, followed by closing tag, and then rest of text
            // unless its the first part, in which case its just the starting unhighlighted bit
            if i > 0 {
                let highlighted_part = part.split(CLOSING_TAG).next().unwrap_or("");
                let remaining_part = part.split(CLOSING_TAG).nth(1).unwrap_or("");
                spans.push(Span::styled(
                    highlighted_part.to_string(),
                    highlighted_snippet_style(),
                ));
                spans.push(Span::styled(
                    remaining_part.to_string(),
                    unhighlighted_snippet_style(),
                ));
            } else {
                spans.push(Span::styled(part, unhighlighted_snippet_style()));
            }
        }
        return Spans::from(spans);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    search: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WikiSearchResponse {
    query: Query,
}

pub fn get_wikipedia_query(
    query: &str,
    shared_caching_session: Shared<CachingSession>,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json",
        query
    );
    let mut caching_session = shared_caching_session.lock().unwrap();

    let query_response: Option<WikiSearchResponse> = match caching_session.has_url(&url) {
        true => {
            // get form cache
            caching_session.get_from_cache::<WikiSearchResponse>(&url)
        }
        false => {
            let client = Client::new();
            let fresh_response = client.get(&url).send()?.json::<WikiSearchResponse>()?;
            caching_session.write_to_cache(&url, &fresh_response)?;
            Some(fresh_response)
        }
    };

    match query_response {
        Some(response) => Ok(response.query.search),
        None => Err("Could not get the query".into()),
    }
}

pub fn load_wikipedia_search_query_to_app(app: &App) {
    if app.input.len() > 0 {
        if app.is_this_lockable() {
            let input = app.input.clone();
            let loading_flag = Arc::clone(&app.is_loading_query);
            let app_results = Arc::clone(&app.results);
            let caching_session = Arc::clone(&app.cache);
            *loading_flag.lock().unwrap() = true;
            thread::spawn(move || {
                if let Ok(results) = get_wikipedia_query(input.as_str(), caching_session) {
                    *app_results.lock().unwrap() = results;
                    *loading_flag.lock().unwrap() = false;
                }
            });
        }
    }
}
