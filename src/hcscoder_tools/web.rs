//! Web search (DuckDuckGo instant API) and HTTP fetch.

use anyhow::{Context, Result};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Web search via DuckDuckGo instant answer API (no API key).
pub async fn web_search(query: &str) -> Result<Vec<SearchResult>> {
    let q = url_encode(query);
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        q
    );
    let client = Client::builder()
        .user_agent(concat!("hcscoder/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("reqwest client")?;

    let text = client
        .get(&url)
        .header("HTTP-Referer", "https://github.com/hcsmediacorp/hcscoder")
        .header("X-Title", "hcscoder")
        .send()
        .await
        .context("DDG request")?
        .text()
        .await
        .context("DDG body")?;

    let v: serde_json::Value = serde_json::from_str(&text).context("parse DDG JSON")?;
    let mut out = Vec::new();

    if let Some(topics) = v.get("RelatedTopics").and_then(|t| t.as_array()) {
        for t in topics.iter().take(15) {
            if let (Some(text), Some(u)) = (
                t.get("Text").and_then(|x| x.as_str()),
                t.get("FirstURL").and_then(|x| x.as_str()),
            ) {
                out.push(SearchResult {
                    title: text.to_string(),
                    url: u.to_string(),
                    snippet: text.to_string(),
                });
            }
        }
    }

    if out.is_empty() {
        out.push(SearchResult {
            title: "No instant results".to_string(),
            url: String::new(),
            snippet: "Try a more specific query or use fetch_url on a known page.".to_string(),
        });
    }
    Ok(out)
}

fn url_encode(q: &str) -> String {
    q.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => {
                let mut s = String::new();
                for b in c.encode_utf8(&mut [0u8; 4]).as_bytes() {
                    s.push_str(&format!("%{:02X}", b));
                }
                s
            }
        })
        .collect()
}

/// Fetch URL body as text (UTF-8 lossy).
pub async fn fetch_url(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent(concat!("hcscoder/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("reqwest client")?;

    let resp = client
        .get(url)
        .header("HTTP-Referer", "https://github.com/hcsmediacorp/hcscoder")
        .header("X-Title", "hcscoder")
        .send()
        .await
        .with_context(|| format!("GET {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} for {}", resp.status(), url);
    }

    let bytes = resp.bytes().await.context("read body")?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}
