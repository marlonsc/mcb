//! Handlebars custom helpers for the admin web UI.
//!
//! Provides formatting, badge rendering, and display helpers that can be
//! called directly from `.hbs` templates.  Register all helpers at once
//! with [`register_helpers`].

use chrono::{DateTime, Utc};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, Renderable,
};
use serde_json::Value;

use crate::constants::display::{
    DEFAULT_ID_TRUNCATE_LENGTH, DEFAULT_TEXT_TRUNCATE_LENGTH, SECS_PER_DAY, SECS_PER_HOUR,
    SECS_PER_MINUTE, SECS_PER_WEEK,
};

/// Formats a Unix-epoch timestamp as `YYYY-MM-DD HH:MM:SS UTC`.
///
/// Usage: `{{timestamp ts}}`
///
/// Renders `"—"` when the value is `0`, null, or missing.
#[derive(Clone, Copy)]
struct TimestampHelper;

impl HelperDef for TimestampHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|p| p.value().as_i64()).unwrap_or(0);

        if val == 0 {
            out.write("\u{2014}")?;
        } else {
            let formatted = DateTime::<Utc>::from_timestamp(val, 0).map_or_else(
                || "\u{2014}".to_owned(),
                |ts| ts.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            );
            out.write(&formatted)?;
        }
        Ok(())
    }
}

/// Renders a human-friendly relative time string from a Unix-epoch timestamp.
///
/// Usage: `{{relative_time ts}}`
///
/// | Delta              | Output            |
/// | -------------------- | ------------------- |
/// | < 60 s             | "just now"        |
/// | < 3 600 s          | "N minutes ago"   |
/// | < 86 400 s         | "N hours ago"     |
/// | < 604 800 s        | "N days ago"      |
/// | >= 604 800 s       | full date string  |
#[derive(Clone, Copy)]
struct RelativeTimeHelper;

impl HelperDef for RelativeTimeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|p| p.value().as_i64()).unwrap_or(0);

        if val == 0 {
            out.write("\u{2014}")?;
            return Ok(());
        }

        let now = Utc::now().timestamp();
        let delta = now.saturating_sub(val);

        let text = if delta < SECS_PER_MINUTE {
            "just now".to_owned()
        } else if delta < SECS_PER_HOUR {
            let mins = delta / SECS_PER_MINUTE;
            format!("{mins} minute{} ago", if mins == 1 { "" } else { "s" })
        } else if delta < SECS_PER_DAY {
            let hours = delta / SECS_PER_HOUR;
            format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
        } else if delta < SECS_PER_WEEK {
            let days = delta / SECS_PER_DAY;
            format!("{days} day{} ago", if days == 1 { "" } else { "s" })
        } else {
            DateTime::<Utc>::from_timestamp(val, 0).map_or_else(
                || "\u{2014}".to_owned(),
                |ts| ts.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            )
        };

        out.write(&text)?;
        Ok(())
    }
}

/// Pretty-prints a JSON value wrapped in `<pre><code>...</code></pre>`.
///
/// Usage: `{{{json_pretty data}}}`  (triple-stache to avoid double-escaping)
///
/// HTML-special characters (`<`, `>`, `&`, `"`) inside the JSON are escaped.
/// Null or missing values render as `<pre><code>null</code></pre>`.
#[derive(Clone, Copy)]
struct JsonPrettyHelper;

impl HelperDef for JsonPrettyHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h.param(0).map_or(Value::Null, |p| p.value().clone());

        let pretty = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_owned());

        let escaped = pretty
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");

        out.write("<pre><code>")?;
        out.write(&escaped)?;
        out.write("</code></pre>")?;
        Ok(())
    }
}

/// Truncates a string to `N` characters (default 8), appending `"..."` when
/// shortened.  Unicode-safe via `.chars().take(n)`.
///
/// Usage: `{{truncate_id id}}` or `{{truncate_id id 12}}`
#[derive(Clone, Copy)]
struct TruncateIdHelper;

impl HelperDef for TruncateIdHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let input = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");

        let max_len = h
            .param(1)
            .and_then(|p| p.value().as_i64())
            .unwrap_or(DEFAULT_ID_TRUNCATE_LENGTH as i64) as usize;

        let char_count = input.chars().count();
        if char_count <= max_len {
            out.write(input)?;
        } else {
            let truncated: String = input.chars().take(max_len).collect();
            out.write(&truncated)?;
            out.write("\u{2026}")?;
        }
        Ok(())
    }
}

/// Returns the singular or plural form of a word based on a count.
///
/// Usage: `{{pluralize count "item" "items"}}`
#[derive(Clone, Copy)]
struct PluralizeHelper;

impl HelperDef for PluralizeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let count = h.param(0).and_then(|p| p.value().as_i64()).unwrap_or(0);

        let singular = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");

        let plural = h.param(2).and_then(|p| p.value().as_str()).unwrap_or("");

        if count == 1 {
            out.write(singular)?;
        } else {
            out.write(plural)?;
        }
        Ok(())
    }
}

/// Renders a generic badge: `<span class="badge badge-{color}">{label}</span>`.
///
/// Usage: `{{{badge label "green"}}}` -- colour defaults to `"gray"`.
#[derive(Clone, Copy)]
struct BadgeHelper;

impl HelperDef for BadgeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let label = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");

        let color = h
            .param(1)
            .and_then(|p| p.value().as_str())
            .unwrap_or("gray");

        out.write(&format!(
            r#"<span class="badge badge-{color}">{label}</span>"#
        ))?;
        Ok(())
    }
}

/// Maps a status string to an auto-coloured badge.
///
/// Usage: `{{{status_badge status}}}`
///
/// | Status                          | Colour  |
/// | --------------------------------- | --------- |
/// | Draft, Stopped                  | gray    |
/// | Active, Running, Healthy        | green   |
/// | InProgress, Starting            | blue    |
/// | Error, Unhealthy                | red     |
/// | Degraded, Stopping              | yellow  |
/// | *(anything else)*               | gray    |
#[derive(Clone, Copy)]
struct StatusBadgeHelper;

impl HelperDef for StatusBadgeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let status = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");

        let color = match status {
            "Draft" | "Stopped" => "gray",
            "Active" | "Running" | "Healthy" => "green",
            "InProgress" | "Starting" => "blue",
            "Error" | "Unhealthy" => "red",
            "Degraded" | "Stopping" => "yellow",
            _ => "gray",
        };

        out.write(&format!(
            r#"<span class="badge badge-{color}">{status}</span>"#
        ))?;
        Ok(())
    }
}

/// Maps a numeric priority (0-4) to a coloured `P0`-`P4` badge.
///
/// Usage: `{{{priority_badge priority}}}`
///
/// | Priority | Label | Colour |
/// | ---------- | ------- | -------- |
/// | 0        | P0    | red    |
/// | 1        | P1    | orange |
/// | 2        | P2    | yellow |
/// | 3        | P3    | blue   |
/// | 4+       | P4    | gray   |
#[derive(Clone, Copy)]
struct PriorityBadgeHelper;

impl HelperDef for PriorityBadgeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let priority = h.param(0).and_then(|p| p.value().as_i64()).unwrap_or(4);

        let color = match priority {
            0 => "red",
            1 => "orange",
            2 => "yellow",
            3 => "blue",
            _ => "gray",
        };

        out.write(&format!(
            r#"<span class="badge badge-{color}">P{priority}</span>"#
        ))?;
        Ok(())
    }
}

/// Truncates text to a maximum length, appending `"..."` when shortened.
///
/// Usage: `{{truncate_text text 60}}`
#[derive(Clone, Copy)]
struct TruncateTextHelper;

impl HelperDef for TruncateTextHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let input = h
            .param(0)
            .map(|p| match p.value() {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();

        let max_len = h
            .param(1)
            .and_then(|p| p.value().as_i64())
            .unwrap_or(DEFAULT_TEXT_TRUNCATE_LENGTH as i64) as usize;

        let char_count = input.chars().count();
        if char_count <= max_len {
            out.write(&input)?;
        } else {
            let truncated: String = input.chars().take(max_len).collect();
            out.write(&truncated)?;
            out.write("\u{2026}")?;
        }
        Ok(())
    }
}

/// Block helper for equality comparison.
///
/// Usage: `{{#eq a b}}matched{{else}}not matched{{/eq}}`
#[derive(Clone, Copy)]
struct EqHelper;

impl HelperDef for EqHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        hbs: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let a = h.param(0).map(handlebars::PathAndJson::value);
        let b = h.param(1).map(handlebars::PathAndJson::value);

        let equal = match (a, b) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };

        if equal {
            if let Some(template) = h.template() {
                template.render(hbs, ctx, rc, out)?;
            }
        } else if let Some(template) = h.inverse() {
            template.render(hbs, ctx, rc, out)?;
        }

        Ok(())
    }
}

/// Register all custom Handlebars helpers on the given registry.
pub fn register_helpers(hbs: &mut Handlebars<'static>) {
    hbs.register_helper("timestamp", Box::new(TimestampHelper));
    hbs.register_helper("relative_time", Box::new(RelativeTimeHelper));
    hbs.register_helper("json_pretty", Box::new(JsonPrettyHelper));
    hbs.register_helper("truncate_id", Box::new(TruncateIdHelper));
    hbs.register_helper("pluralize", Box::new(PluralizeHelper));
    hbs.register_helper("badge", Box::new(BadgeHelper));
    hbs.register_helper("status_badge", Box::new(StatusBadgeHelper));
    hbs.register_helper("priority_badge", Box::new(PriorityBadgeHelper));
    hbs.register_helper("truncate_text", Box::new(TruncateTextHelper));
    hbs.register_helper("eq", Box::new(EqHelper));
}
