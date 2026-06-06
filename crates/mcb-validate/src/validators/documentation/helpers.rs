//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use regex::Regex;

pub(super) struct DocItemContext<'a> {
    pub(super) path: &'a std::path::Path,
    pub(super) lines: &'a [&'a str],
    pub(super) line_num: usize,
    pub(super) item_name: &'a str,
}

pub(super) struct DocRegexContext<'a> {
    pub(super) doc_comment_re: &'a Regex,
    pub(super) doc_comment_capture_re: &'a Regex,
    pub(super) attr_re: &'a Regex,
    pub(super) example_pattern: &'a Regex,
}

pub(super) struct MissingDocSpec<'a> {
    pub(super) item_kind: &'a str,
    pub(super) severity: crate::Severity,
}

pub(super) struct SimplePubItemSpec<'a> {
    pub(super) pattern: &'a Regex,
    pub(super) item_kind: &'a str,
}

pub(super) struct ScanLineContext<'a> {
    pub(super) path: &'a std::path::Path,
    pub(super) lines: &'a [&'a str],
    pub(super) line_num: usize,
    pub(super) line: &'a str,
}

pub(super) fn has_doc_comment(
    lines: &[&str],
    item_line: usize,
    doc_re: &Regex,
    attr_re: &Regex,
) -> bool {
    if item_line == 0 {
        return false;
    }

    let bracket_delta = |raw: &str| -> i32 {
        let closes = (raw.matches(']').count() + raw.matches(')').count()) as i32;
        let opens = (raw.matches('[').count() + raw.matches('(').count()) as i32;
        closes - opens
    };

    let mut i = item_line - 1;
    // Depth of an in-progress multi-line attribute block (e.g. a wrapped
    // `#[derive(\n ... \n)]`) encountered while scanning upward.
    let mut attr_depth: i32 = 0;
    loop {
        let raw = lines[i];
        let line = raw.trim();

        // Inside a multi-line attribute: consume lines until its `#[` opener.
        if attr_depth > 0 {
            attr_depth += bracket_delta(raw);
        } else if line.is_empty() || attr_re.is_match(raw) {
            // blank line or single-line attribute — keep scanning upward
        } else if line.ends_with(']') && bracket_delta(raw) > 0 {
            // closing line of a multi-line attribute (e.g. `)]`)
            attr_depth = bracket_delta(raw);
        } else {
            return doc_re.is_match(raw);
        }

        if i == 0 {
            return false;
        }
        i -= 1;
    }
}

pub(super) fn get_doc_comment_section(
    lines: &[&str],
    item_line: usize,
    doc_capture_re: &Regex,
    attr_re: &Regex,
) -> String {
    if item_line == 0 {
        return String::new();
    }

    let mut doc_lines = Vec::new();
    let mut i = item_line - 1;

    loop {
        let line = lines[i];
        if attr_re.is_match(line) {
            if i == 0 {
                break;
            }
            i -= 1;
            continue;
        }

        if let Some(cap) = doc_capture_re.captures(line) {
            let content = cap.get(1).map_or("", |m| m.as_str());
            doc_lines.push(content);
        } else if !line.trim().is_empty() {
            break;
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    doc_lines.reverse();
    doc_lines.join("\n")
}
