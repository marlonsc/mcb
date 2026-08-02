//! Search result formatting.

use std::fmt::Write;
use std::path::Path;
use std::time::Duration;

use mcb_domain::value_objects::SearchResult;
use mcb_utils::constants::display::CODE_PREVIEW_MAX_LINES;
use mcb_utils::constants::search::SEARCH_SLOW_THRESHOLD_MS;

pub(super) fn build_search_response_message(
    query: &str,
    results: &[SearchResult],
    duration: Duration,
    limit: usize,
) -> String {
    let mut message = "🔍 **Semantic Code Search Results**\n\n".to_owned();
    let _ = writeln!(message, "**Query:** \"{query}\" ");
    let _ = writeln!(
        message,
        "**Search completed in:** {:.2}s",
        duration.as_secs_f64()
    );
    let _ = write!(message, "**Results found:** {}\n\n", results.len());

    if results.is_empty() {
        append_empty_search_response(&mut message);
    } else {
        append_search_results(&mut message, results, limit, duration);
    }

    message
}

fn append_empty_search_response(message: &mut String) {
    message.push_str("❌ **No Results Found**\n\n");
    message.push_str("**Possible Reasons:**\n");
    message.push_str("• Codebase not indexed yet (run `index` action=start first)\n");
    message.push_str("• Query terms not present in the codebase\n");
    message.push_str("• Try different keywords or more general terms\n\n");
    message.push_str("**Search Tips:**\n");
    message.push_str("• Use natural language: \"find error handling\", \"authentication logic\"\n");
    message.push_str("• Be specific: \"HTTP request middleware\" > \"middleware\"\n");
    message.push_str("• Include technologies: \"React component state management\"\n");
    message.push_str("• Try synonyms: \"validate\" instead of \"check\"\n");
}

fn append_search_results(
    message: &mut String,
    results: &[SearchResult],
    limit: usize,
    duration: Duration,
) {
    message.push_str("📊 **Search Results:**\n\n");

    for (i, result) in results.iter().enumerate() {
        let _ = writeln!(
            message,
            "**{}.** 📁 `{}` (line {})",
            i + 1,
            result.file_path,
            result.start_line
        );

        append_code_preview(message, result);
        let _ = write!(message, "🎯 **Relevance Score:** {:.3}\n\n", result.score);
    }

    if results.len() == limit {
        let _ = writeln!(
            message,
            "💡 **Showing top {limit} results.** For more results, try:"
        );
        message.push_str("• More specific search terms\n");
        message.push_str("• Different query formulations\n");
        message.push_str("• Breaking complex queries into simpler ones\n");
    }

    if duration.as_millis() > SEARCH_SLOW_THRESHOLD_MS {
        let _ = write!(
            message,
            "\n⚠️ **Performance Note:** Search took {:.2}s. Consider using more specific queries for faster results.\n",
            duration.as_secs_f64()
        );
    }
}

fn append_code_preview(message: &mut String, result: &SearchResult) {
    let lines: Vec<&str> = result.content.lines().collect();
    let preview_lines = if lines.len() > CODE_PREVIEW_MAX_LINES {
        lines
            .iter()
            .take(CODE_PREVIEW_MAX_LINES)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        result.content.clone()
    };

    let file_ext = Path::new(&result.file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let lang_hint = get_language_hint(file_ext, &result.language);

    if lang_hint.is_empty() {
        let _ = write!(message, "```\n{preview_lines}\n```\n");
    } else {
        let _ = write!(message, "``` {lang_hint}\n{preview_lines}\n```\n");
    }
}

fn get_language_hint<'a>(file_ext: &str, default_lang: &'a str) -> &'a str {
    match file_ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "go" => "go",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "cs" => "csharp",
        _ => default_lang,
    }
}
