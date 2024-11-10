use reqwest::blocking::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{error::Error, fs::File};
use tui::text::{Span, Spans};

use crate::styles::{highlighted_snippet_style, unhighlighted_snippet_style};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
}

const OPENING_TAG: &str = "<span class=\"searchmatch\">";
const CLOSING_TAG: &str = "</span>";

impl SearchResult {
    pub fn highlighted_snippets(search_results: &SearchResult) -> Spans {
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

pub fn get_wikipedia_query(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    // get from retrieve or make query

    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json",
        query
    );
    let client = Client::new();
    // let response = client.get(&url).send()?;
    // serde_json::to_writer_pretty(writer, value)
    let response = client.get(&url).send()?.json::<WikiSearchResponse>()?;

    Ok(response.query.search)
}
