//! HTML naar platte tekst voor onderzoeks- en schrijfpipeline (`html2text`, met regex-fallback).

use std::io::Cursor;
use std::sync::OnceLock;

use regex::Regex;

/// Haalt markup uit HTML-heavy responses; bij parsefout een lichte regex-strip.
///
/// Platte invoer blijft in de praktijk leesbaar (geen echte `<`-tags ⇒ weinig verschil).
pub fn strip_html_to_plain(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match html2text::from_read(Cursor::new(trimmed.as_bytes()), 120) {
        Ok(s) => collapse_excess_blank_lines(&s),
        Err(e) => {
            tracing::debug!(
                error = %e,
                "html2text mislukt; regex-fallback voor HTML-strip"
            );
            collapse_excess_blank_lines(&regex_strip_markup(trimmed))
        }
    }
}

fn collapse_excess_blank_lines(s: &str) -> String {
    static MULTI_NL: OnceLock<Regex> = OnceLock::new();
    let re = MULTI_NL.get_or_init(|| Regex::new(r"\n{3,}").expect("MULTI_NL"));
    let t = re.replace_all(s.trim_end(), "\n\n");
    t.into_owned()
}

fn regex_strip_markup(s: &str) -> String {
    static SCRIPT: OnceLock<Regex> = OnceLock::new();
    static STYLE: OnceLock<Regex> = OnceLock::new();
    static TAGS: OnceLock<Regex> = OnceLock::new();

    let step1 = SCRIPT
        .get_or_init(|| Regex::new(r"(?si)<script[^>]*>.*?</script>").expect("SCRIPT"))
        .replace_all(s, "");
    let step2 = STYLE
        .get_or_init(|| Regex::new(r"(?si)<style[^>]*>.*?</style>").expect("STYLE"))
        .replace_all(&step1, "");
    let step3 = TAGS
        .get_or_init(|| Regex::new(r"<[^>]+>").expect("TAGS"))
        .replace_all(&step2, " ");

    let t = html_entity_light(&step3);
    t.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn html_entity_light(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_simple_tag() {
        let t = strip_html_to_plain("<p>Hello <b>world</b></p>");
        assert!(t.contains("Hello"));
        assert!(t.contains("world"));
        assert!(!t.contains("<p>"));
    }
}
