use mcb_domain::value_objects::browse::{HighlightCategory, HighlightedCode};

/// Renders highlighted code to HTML.
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Converts a `HighlightedCode` object into an HTML string with CSS classes.
    #[must_use]
    pub fn render(highlighted: &HighlightedCode) -> String {
        if highlighted.original.is_empty() {
            return String::new();
        }

        let mut html = String::new();
        let mut last_end = 0;

        let mut sorted_spans = highlighted.spans.clone();
        sorted_spans.sort_by_key(|s| s.start);

        for span in sorted_spans {
            if last_end < span.start {
                let text = &highlighted.original[last_end..span.start];
                html.push_str(&Self::html_escape(text));
            }

            let class = Self::category_to_css_class(span.category);
            let text = &highlighted.original[span.start..span.end];
            html.push_str(&format!(
                "<span class=\"{}\">{}</span>",
                class,
                Self::html_escape(text)
            ));

            last_end = span.end;
        }

        if last_end < highlighted.original.len() {
            let text = &highlighted.original[last_end..];
            html.push_str(&Self::html_escape(text));
        }

        html
    }

    fn category_to_css_class(category: HighlightCategory) -> &'static str {
        match category {
            HighlightCategory::Keyword => "hl-keyword",
            HighlightCategory::String => "hl-string",
            HighlightCategory::Comment => "hl-comment",
            HighlightCategory::Function => "hl-function",
            HighlightCategory::Variable => "hl-variable",
            HighlightCategory::Type => "hl-type",
            HighlightCategory::Number => "hl-number",
            HighlightCategory::Operator => "hl-operator",
            HighlightCategory::Punctuation => "hl-punctuation",
            HighlightCategory::Other => "hl-other",
        }
    }

    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
