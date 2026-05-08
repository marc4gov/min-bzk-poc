//! Optionele echte HTTP-GET voor [`ResearchExpert`]. Alleen aan met `MESH_RESEARCH_HTTP=1` / `true` / `yes`.
//! Body wordt naar platte tekst omgezet ([`crate::mesh::html_plain::strip_html_to_plain`]) en daarna ingekort.

use std::sync::OnceLock;
use std::time::Duration;

use regex::Regex;

use crate::mesh::html_plain::strip_html_to_plain;

/// Milieuvariabele om fetch aan te zetten.
pub const ENV_RESEARCH_HTTP: &str = "MESH_RESEARCH_HTTP";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_BYTES_PER_URL: usize = 48 * 1024;
const PROMPT_CHARS: usize = 12_000;

pub fn http_fetch_enabled() -> bool {
    matches!(
        std::env::var(ENV_RESEARCH_HTTP).as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn url_find_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://\S+").expect("url regex"))
}

/// Alle `http://`/`https://`-URLs in vrije tekst (bv. gebruikersvraag met link erin).
pub fn extract_http_urls_from_text(query: &str) -> Vec<String> {
    let re = url_find_regex();
    re.find_iter(query)
        .map(|m| {
            m.as_str()
                .trim_end_matches(|c| matches!(c, '.' | ',' | ';' | ':' | '!'))
                .to_string()
        })
        .collect()
}

fn validate_http_url(raw: &str) -> Result<(), String> {
    let t = raw.trim();
    let lower = t.to_ascii_lowercase();
    if !lower.starts_with("https://") && !lower.starts_with("http://") {
        return Err("alleen http(s)-URLs".into());
    }
    Ok(())
}

/// Haalt waar mogelijk `http(s)`-URLs op als UTF-8 (lossy). Is een item geen geldige URL, dan
/// wordt die bron overgeslagen met uitleg; andere bronnen lopen door (één slechte string verpest de batch niet).
pub async fn fetch_research_notes(urls: &[String], depth_hint: u8) -> Result<String, String> {
    if urls.is_empty() {
        return Err("geen URLs meegegeven".into());
    }

    let client = reqwest::Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .user_agent("LocalAssistant-agent-service/research-http")
        .build()
        .map_err(|e| e.to_string())?;

    let chunk_budget = (PROMPT_CHARS / urls.len().max(1)).max(500);

    let mut sections = Vec::new();
    for (i, raw_url) in urls.iter().enumerate() {
        let label = raw_url.trim().to_string();
        if validate_http_url(raw_url).is_err() {
            sections.push(format!(
                "### Bron {} (overgeslagen — geen http(s)-URL)\nRuwe inhoud: {}\n_diepte-parameter: {}_\n",
                i + 1,
                label,
                depth_hint
            ));
            continue;
        }

        let url_to_get = raw_url.trim();
        let resp_result = client.get(url_to_get).send().await;

        let section = match resp_result {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    format!(
                        "### Bron {} (HTTP {})\nURL: {}\n---\n(geen succes-response; inhoud niet geladen)\n",
                        i + 1,
                        status.as_u16(),
                        url_to_get
                    )
                } else {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            let take = MAX_BYTES_PER_URL.min(bytes.len());
                            let slice = &bytes[..take];
                            let text = String::from_utf8_lossy(slice);
                            let plain = strip_html_to_plain(text.as_ref());
                            format!(
                                "### Bron {} (diepte-hint: {})\nURL: {}\n---\n{}\n",
                                i + 1,
                                depth_hint,
                                url_to_get,
                                clamp_chars(&plain, chunk_budget)
                            )
                        }
                        Err(e) => format!(
                            "### Bron {} (body-fout)\nURL: {}\n---\n{e}\n",
                            i + 1,
                            url_to_get
                        ),
                    }
                }
            }
            Err(e) => format!(
                "### Bron {} (netwerkfout)\nURL: {}\n---\n{e}\n",
                i + 1,
                url_to_get
            ),
        };

        sections.push(section);
    }

    Ok(format!(
        "ResearchExpert (HTTP-preview; lossy UTF-8):\n\n{}",
        sections.join("\n")
    ))
}

fn clamp_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let omit = count.saturating_sub(max.saturating_sub(1));
    let prefix: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{prefix}… (+{omit} tekens niet getoond)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_scheme() {
        assert!(validate_http_url("ftp://example.com").is_err());
    }

    #[test]
    fn accepts_https() {
        assert!(validate_http_url("https://example.com/path").is_ok());
    }

    #[test]
    fn extracts_two_urls() {
        let q = "See https://example.org/a en https://rust-lang.org";
        let v = extract_http_urls_from_text(q);
        assert!(v.len() >= 2);
        assert!(v.iter().any(|u| u.contains("example.org")));
    }
}
