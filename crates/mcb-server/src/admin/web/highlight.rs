use html_escape::{encode_double_quoted_attribute, encode_text};
use std::fmt::Write;
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

const HIGHLIGHT_NAMES: [&str; 13] = [
    "keyword",
    "function",
    "string",
    "comment",
    "type",
    "variable",
    "constant",
    "operator",
    "attribute",
    "number",
    "punctuation",
    "property",
    "tag",
];

pub fn highlight_code(code: &str, language: &str) -> String {
    if code.is_empty() {
        return String::new();
    }

    let highlighted = highlight_html(code, language).unwrap_or_else(|| plain_text_html(code));
    wrap_lines(&highlighted)
}

pub fn highlight_chunks(chunks: &[(String, u32, u32, String)], language: &str) -> String {
    let mut output = String::new();

    for (content, start_line, end_line, chunk_id) in chunks {
        let highlighted = highlight_code(content, language);
        let safe_chunk_id = encode_double_quoted_attribute(chunk_id);

        let _ = write!(
            output,
            "<div class=\"chunk\" data-chunk-start=\"{}\" data-chunk-end=\"{}\" data-chunk-id=\"{}\">",
            start_line, end_line, safe_chunk_id
        );
        let _ = write!(
            output,
            "<div class=\"chunk-header\">Lines {}-{}</div>",
            start_line, end_line
        );
        output.push_str("<pre class=\"chunk-code\"><code>");
        output.push_str(&highlighted);
        output.push_str("</code></pre></div>");
    }

    output
}

fn highlight_html(code: &str, language: &str) -> Option<String> {
    let config = highlight_configuration(language)?;
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |_| None)
        .ok()?;

    let mut output = String::new();
    let mut open_spans: Vec<&str> = Vec::new();

    for event in highlights {
        let event = match event {
            Ok(event) => event,
            Err(_) => return None,
        };

        match event {
            HighlightEvent::Source { start, end } => {
                let fragment = &code[start..end];
                output.push_str(&encode_text(fragment));
            }
            HighlightEvent::HighlightStart(Highlight(highlight)) => {
                if let Some(class_name) = HIGHLIGHT_NAMES.get(highlight) {
                    output.push_str("<span class=\"hl-");
                    output.push_str(class_name);
                    output.push_str("\">");
                    open_spans.push(class_name);
                }
            }
            HighlightEvent::HighlightEnd => {
                if open_spans.pop().is_some() {
                    output.push_str("</span>");
                }
            }
        }
    }

    while open_spans.pop().is_some() {
        output.push_str("</span>");
    }

    Some(output)
}

fn highlight_configuration(language: &str) -> Option<HighlightConfiguration> {
    let config = language_config(language)?;
    let mut config = HighlightConfiguration::new(
        config.language,
        config.name,
        config.highlights_query,
        "",
        "",
    )
    .ok()?;
    config.configure(&HIGHLIGHT_NAMES);
    Some(config)
}

fn language_config(language: &str) -> Option<LanguageConfig> {
    let normalized = language.trim().to_lowercase();

    match normalized.as_str() {
        "rust" => Some(LanguageConfig::new(
            "rust",
            tree_sitter_rust::LANGUAGE.into(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
        )),
        "python" => Some(LanguageConfig::new(
            "python",
            tree_sitter_python::LANGUAGE.into(),
            tree_sitter_python::HIGHLIGHTS_QUERY,
        )),
        "javascript" | "js" => Some(LanguageConfig::new(
            "javascript",
            tree_sitter_javascript::LANGUAGE.into(),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
        )),
        "typescript" | "ts" => Some(LanguageConfig::new(
            "typescript",
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        )),
        "tsx" => Some(LanguageConfig::new(
            "tsx",
            tree_sitter_typescript::LANGUAGE_TSX.into(),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
        )),
        "go" => Some(LanguageConfig::new(
            "go",
            tree_sitter_go::LANGUAGE.into(),
            tree_sitter_go::HIGHLIGHTS_QUERY,
        )),
        "java" => Some(LanguageConfig::new(
            "java",
            tree_sitter_java::LANGUAGE.into(),
            tree_sitter_java::HIGHLIGHTS_QUERY,
        )),
        "c" => Some(LanguageConfig::new(
            "c",
            tree_sitter_c::LANGUAGE.into(),
            tree_sitter_c::HIGHLIGHT_QUERY,
        )),
        "cpp" | "c++" => Some(LanguageConfig::new(
            "cpp",
            tree_sitter_cpp::LANGUAGE.into(),
            tree_sitter_cpp::HIGHLIGHT_QUERY,
        )),
        "ruby" => Some(LanguageConfig::new(
            "ruby",
            tree_sitter_ruby::LANGUAGE.into(),
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
        )),
        "php" => Some(LanguageConfig::new(
            "php",
            tree_sitter_php::LANGUAGE_PHP.into(),
            tree_sitter_php::HIGHLIGHTS_QUERY,
        )),
        "swift" => Some(LanguageConfig::new(
            "swift",
            tree_sitter_swift::LANGUAGE.into(),
            tree_sitter_swift::HIGHLIGHTS_QUERY,
        )),
        _ => None,
    }
}

struct LanguageConfig {
    language: Language,
    name: &'static str,
    highlights_query: &'static str,
}

impl LanguageConfig {
    fn new(name: &'static str, language: Language, highlights_query: &'static str) -> Self {
        Self {
            language,
            name,
            highlights_query,
        }
    }
}

fn wrap_lines(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let mut line_number = 1;
    let mut lines = html.split('\n').peekable();

    while let Some(line) = lines.next() {
        let _ = write!(
            output,
            "<span class=\"line\" data-line=\"{}\">",
            line_number
        );
        output.push_str(line);
        output.push_str("</span>");

        if lines.peek().is_some() {
            output.push('\n');
        }

        line_number += 1;
    }

    output
}

fn plain_text_html(code: &str) -> String {
    encode_text(code).into_owned()
}

#[cfg(test)]
mod tests {
    use super::highlight_code;

    #[test]
    fn highlights_rust_keyword() {
        let html = highlight_code("fn main() {}", "rust");
        assert!(html.contains("hl-keyword"), "unexpected html: {html}");
    }

    #[test]
    fn highlights_python_number() {
        let html = highlight_code("x = 1", "python");
        assert!(html.contains("hl-number"), "unexpected html: {html}");
    }

    #[test]
    fn falls_back_for_unknown_language() {
        let html = highlight_code("unknown", "brainfuck");
        assert!(html.contains("unknown"));
        assert!(!html.contains("hl-"), "unexpected html: {html}");
    }

    #[test]
    fn handles_empty_input() {
        let html = highlight_code("", "rust");
        assert!(html.is_empty());
    }
}
