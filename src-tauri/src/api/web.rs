// Web access tools exposed to the AI agent.
//
// `web_fetch` retrieves a single URL and returns readable text (HTML is stripped
// to plain text); `web_search` runs a keyless DuckDuckGo query and returns the
// top hits. Both run on the Rust side via reqwest, so they are not subject to the
// webview's CORS policy and give the agent the same "look something up" ability
// that Claude Code / Codex have. Network data (weather, docs, news) flows in here.

use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

const DEFAULT_MAX_CHARS: usize = 8000;
const REQUEST_TIMEOUT_SECS: u64 = 20;
// A real browser-ish UA: some sites (and DuckDuckGo's HTML endpoint) serve empty
// or blocked responses to obvious bots.
const USER_AGENT: &str =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
   (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

fn http_client() -> Result<Client, String> {
  Client::builder()
    .user_agent(USER_AGENT)
    .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
    .build()
    .map_err(|e| e.to_string())
}

// ==================== web_fetch ====================

#[derive(Debug, Serialize)]
pub struct WebFetchResult {
  pub url: String,
  pub status: u16,
  pub content_type: String,
  pub title: Option<String>,
  pub text: String,
  pub truncated: bool,
}

/// Fetch a single URL. HTML responses are reduced to readable text (scripts /
/// styles / tags removed, whitespace collapsed); other text responses are
/// returned as-is. The body is capped at `max_chars` so a huge page does not
/// blow up the model's context.
#[tauri::command]
pub async fn web_fetch(url: String, max_chars: Option<usize>) -> Result<WebFetchResult, String> {
  if !(url.starts_with("http://") || url.starts_with("https://")) {
    return Err("URL must start with http:// or https://".into());
  }

  let client = http_client()?;
  let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
  let status = res.status().as_u16();
  let content_type = res
    .headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_string();

  let body = res.text().await.map_err(|e| e.to_string())?;
  let looks_html = content_type.contains("text/html") || body.trim_start().starts_with('<');

  let (title, mut text) = if looks_html {
    (extract_title(&body), html_to_text(&body))
  } else {
    (None, body)
  };

  let limit = max_chars.unwrap_or(DEFAULT_MAX_CHARS).clamp(200, 50_000);
  let truncated = text.chars().count() > limit;
  if truncated {
    text = text.chars().take(limit).collect();
  }

  Ok(WebFetchResult {
    url,
    status,
    content_type,
    title,
    text,
    truncated,
  })
}

// ==================== web_search ====================

#[derive(Debug, Serialize)]
pub struct SearchHit {
  pub title: String,
  pub url: String,
  pub snippet: String,
}

/// Keyless web search via DuckDuckGo's HTML endpoint. Returns up to
/// `max_results` hits (title / resolved URL / snippet). Best-effort: if the
/// markup changes and nothing parses, this errors so the caller can fall back.
#[tauri::command]
pub async fn web_search(
  query: String,
  max_results: Option<usize>,
) -> Result<Vec<SearchHit>, String> {
  let q = query.trim();
  if q.is_empty() {
    return Err("search query is empty".into());
  }

  let client = http_client()?;
  let res = client
    .get("https://html.duckduckgo.com/html/")
    .query(&[("q", q)])
    .send()
    .await
    .map_err(|e| e.to_string())?;
  let html = res.text().await.map_err(|e| e.to_string())?;

  let limit = max_results.unwrap_or(5).clamp(1, 10);
  let link_re =
    regex::Regex::new(r#"(?is)<a[^>]*class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)
      .map_err(|e| e.to_string())?;
  let snippet_re =
    regex::Regex::new(r#"(?is)<a[^>]*class="result__snippet"[^>]*>(.*?)</a>"#)
      .map_err(|e| e.to_string())?;

  let snippets: Vec<String> = snippet_re
    .captures_iter(&html)
    .map(|c| strip_tags(&c[1]))
    .collect();

  let mut hits = Vec::new();
  for (i, cap) in link_re.captures_iter(&html).enumerate() {
    if hits.len() >= limit {
      break;
    }
    hits.push(SearchHit {
      title: strip_tags(&cap[2]),
      url: extract_ddg_target(&cap[1]),
      snippet: snippets.get(i).cloned().unwrap_or_default(),
    });
  }

  if hits.is_empty() {
    return Err("No results parsed (DuckDuckGo markup may have changed)".into());
  }
  Ok(hits)
}

// ==================== HTML / URL helpers ====================

fn extract_title(html: &str) -> Option<String> {
  let re = regex::Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok()?;
  re.captures(html)
    .and_then(|c| c.get(1))
    .map(|m| decode_entities(m.as_str()).trim().to_string())
    .filter(|t| !t.is_empty())
}

/// Strip `<script>` / `<style>` blocks and all tags, decode common entities, and
/// collapse runs of whitespace into single spaces.
fn html_to_text(html: &str) -> String {
  let drop_blocks = regex::Regex::new(r"(?is)<(script|style|noscript|head)[^>]*>.*?</\s*\1\s*>")
    .map(|re| re.replace_all(html, " ").into_owned())
    .unwrap_or_else(|_| html.to_string());
  strip_tags(&drop_blocks)
}

/// Remove all HTML tags from a fragment, decode entities, collapse whitespace.
fn strip_tags(fragment: &str) -> String {
  let no_tags = regex::Regex::new(r"(?s)<[^>]+>")
    .map(|re| re.replace_all(fragment, " ").into_owned())
    .unwrap_or_else(|_| fragment.to_string());
  let decoded = decode_entities(&no_tags);
  regex::Regex::new(r"\s+")
    .map(|re| re.replace_all(&decoded, " ").trim().to_string())
    .unwrap_or_else(|_| decoded.trim().to_string())
}

fn decode_entities(s: &str) -> String {
  s.replace("&nbsp;", " ")
    .replace("&amp;", "&")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'")
    .replace("&#x27;", "'")
}

/// DuckDuckGo wraps result links as `//duckduckgo.com/l/?uddg=<encoded>&...`.
/// Pull out and percent-decode the real target; fall back to the href as-is.
fn extract_ddg_target(href: &str) -> String {
  if let Some(idx) = href.find("uddg=") {
    let rest = &href[idx + "uddg=".len()..];
    let end = rest.find('&').unwrap_or(rest.len());
    return percent_decode(&rest[..end]);
  }
  if let Some(stripped) = href.strip_prefix("//") {
    format!("https://{}", stripped)
  } else {
    href.to_string()
  }
}

fn percent_decode(s: &str) -> String {
  let bytes = s.as_bytes();
  let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'%' if i + 2 < bytes.len() => {
        match (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
          (Some(hi), Some(lo)) => {
            out.push(hi * 16 + lo);
            i += 3;
          }
          _ => {
            out.push(bytes[i]);
            i += 1;
          }
        }
      }
      b'+' => {
        out.push(b' ');
        i += 1;
      }
      c => {
        out.push(c);
        i += 1;
      }
    }
  }
  String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
  match b {
    b'0'..=b'9' => Some(b - b'0'),
    b'a'..=b'f' => Some(b - b'a' + 10),
    b'A'..=b'F' => Some(b - b'A' + 10),
    _ => None,
  }
}
