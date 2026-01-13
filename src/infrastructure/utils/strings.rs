//! String utilities - Common string operations (DRY)

/// String utilities - common string operations
pub struct StringUtils;

impl StringUtils {
    /// Capitalize first letter of a string
    #[inline]
    pub fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }

    /// Convert snake_case or space-separated string to Title Case
    ///
    /// # Examples
    /// - "open_ai" → "Open Ai"
    /// - "hello world" → "Hello World"
    /// - "my_api_key" → "My Api Key"
    pub fn to_title_case(s: &str) -> String {
        s.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format relative time from chrono DateTime (e.g., "Just now", "5m ago")
    pub fn format_relative_time(timestamp: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(timestamp);
        let seconds = diff.num_seconds();

        const SECONDS_PER_MINUTE: i64 = 60;
        const SECONDS_PER_HOUR: i64 = 3600;
        const SECONDS_PER_DAY: i64 = 86400;

        if seconds < SECONDS_PER_MINUTE {
            "Just now".to_string()
        } else if seconds < SECONDS_PER_HOUR {
            format!("{}m ago", seconds / SECONDS_PER_MINUTE)
        } else if seconds < SECONDS_PER_DAY {
            format!("{}h ago", seconds / SECONDS_PER_HOUR)
        } else {
            format!("{}d ago", seconds / SECONDS_PER_DAY)
        }
    }
}
