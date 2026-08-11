use scraper::{Html, Selector};
use url::Url;

pub struct FetchedContent {
    pub url: String,
    pub title: String,
    pub body: String,
    pub byte_size: usize,
}

pub fn fetch_and_extract(raw_url: &str) -> Result<FetchedContent, String> {
    let url = Url::parse(raw_url).map_err(|e| format!("Invalid URL: {}", e))?;

    let html = reqwest::blocking::get(url.as_str())
        .map_err(|e| format!("Fetch failed: {}", e))?
        .text()
        .map_err(|e| format!("Read failed: {}", e))?;

    let document = Html::parse_document(&html);

    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".to_string());

    let body_selector = Selector::parse("article, main, .content, .page-content, p, li, h1, h2, h3")
        .unwrap();
    let body: String = document
        .select(&body_selector)
        .flat_map(|el| el.text())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Ok(FetchedContent {
        url: raw_url.to_string(),
        title: title.trim().to_string(),
        body: body.trim().to_string(),
        byte_size: body.len(),
    })
}

pub fn parse_read_command(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.starts_with("[read:") && trimmed.ends_with(']') {
        let url = &trimmed[6..trimmed.len() - 1];
        Some(url.trim().to_string())
    } else {
        None
    }
}
