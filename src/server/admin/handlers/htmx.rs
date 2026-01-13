//! HTMX partial response handlers

use super::common::*;
use crate::infrastructure::utils::css;

/// HTMX partial for recovery status
pub async fn htmx_recovery_status_handler(State(state): State<AdminState>) -> Html<String> {
    let recovery_states = if let Some(recovery_manager) = &state.recovery_manager {
        recovery_manager.get_recovery_states()
    } else {
        Vec::new()
    };

    if recovery_states.is_empty() {
        return Html(
            r#"<div class="px-6 py-4 text-center text-gray-500 dark:text-gray-400">
                <svg class="h-8 w-8 mx-auto text-green-500 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                All subsystems healthy - no recovery in progress
            </div>"#
                .to_string(),
        );
    }

    let mut html = String::from(r#"<div class="divide-y divide-gray-200 dark:divide-gray-700">"#);

    for state in recovery_states {
        let status_color = match state.status {
            crate::daemon::types::RecoveryStatus::Healthy => "bg-green-500",
            crate::daemon::types::RecoveryStatus::Recovering => "bg-yellow-500 animate-pulse",
            crate::daemon::types::RecoveryStatus::Exhausted => "bg-red-500",
            crate::daemon::types::RecoveryStatus::Manual => "bg-orange-500",
        };

        let status_text = format!("{}", state.status);

        html.push_str(&format!(
            r#"<div class="flex items-center justify-between px-6 py-4">
                <div class="flex items-center space-x-3">
                    <span class="w-3 h-3 rounded-full {}"></span>
                    <div>
                        <span class="font-medium text-gray-900 dark:text-white">{}</span>
                        <span class="ml-2 text-sm text-gray-500 dark:text-gray-400">{}</span>
                    </div>
                </div>
                <div class="flex items-center space-x-4">"#,
            status_color, state.subsystem_id, status_text
        ));

        if state.current_retry > 0 {
            html.push_str(&format!(
                r#"<span class="text-sm text-gray-500">Retry {}/{}</span>"#,
                state.current_retry, state.max_retries
            ));
        }

        if let Some(ref error) = state.last_error {
            html.push_str(&format!(
                r#"<span class="text-xs text-red-500 max-w-xs truncate" title="{}">{}</span>"#,
                error,
                if error.len() > 30 {
                    format!("{}...", &error[..30])
                } else {
                    error.clone()
                }
            ));
        }

        // Action buttons based on status
        if state.status == crate::daemon::types::RecoveryStatus::Exhausted {
            html.push_str(&format!(
                r##"<button hx-post="/admin/api/recovery/{}/reset"
                          hx-target="#recovery-status"
                          hx-swap="innerHTML"
                          class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300">
                    Reset
                </button>"##,
                state.subsystem_id
            ));
        }

        html.push_str(&format!(
            r##"<button hx-post="/admin/api/recovery/{}/trigger"
                      hx-target="#recovery-status"
                      hx-swap="innerHTML"
                      class="text-sm text-orange-600 hover:text-orange-800 dark:text-orange-400 dark:hover:text-orange-300">
                Retry Now
            </button>"##,
            state.subsystem_id
        ));

        html.push_str("</div></div>");
    }

    html.push_str("</div>");

    Html(html)
}

/// HTMX partial for maintenance history
pub async fn htmx_maintenance_history_handler(State(state): State<AdminState>) -> Html<String> {
    let activities = state.activity_logger.get_activities(Some(10)).await;

    if activities.is_empty() {
        return Html(
            r#"<div class="px-6 py-4">
                <div class="text-center text-gray-500 dark:text-gray-400 py-4">
                    <svg class="h-8 w-8 mx-auto text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    <p>No recent maintenance activity.</p>
                    <p class="text-sm mt-1">Operations will appear here as they occur.</p>
                </div>
            </div>"#
                .to_string(),
        );
    }

    let mut html = String::from(r#"<div class="divide-y divide-gray-200 dark:divide-gray-700">"#);

    for activity in activities {
        let level_class = match activity.level {
            crate::admin::service::helpers::activity::ActivityLevel::Info => css::badge::INFO,
            crate::admin::service::helpers::activity::ActivityLevel::Warning => css::badge::WARNING,
            crate::admin::service::helpers::activity::ActivityLevel::Error => css::badge::ERROR,
            crate::admin::service::helpers::activity::ActivityLevel::Success => css::badge::SUCCESS,
        };

        let time_ago = chrono::Utc::now()
            .signed_duration_since(activity.timestamp)
            .num_seconds();
        let time_str = if time_ago < 60 {
            format!("{}s ago", time_ago)
        } else if time_ago < 3600 {
            format!("{}m ago", time_ago / 60)
        } else {
            format!("{}h ago", time_ago / 3600)
        };

        html.push_str(&format!(
            r#"<div class="px-4 py-3">
                <div class="flex items-center justify-between">
                    <span class="text-xs font-medium px-2 py-1 rounded {}">{}</span>
                    <span class="text-xs text-gray-500">{}</span>
                </div>
                <p class="mt-1 text-sm text-gray-900 dark:text-gray-100">{}</p>
            </div>"#,
            level_class, activity.category, time_str, activity.message
        ));
    }

    html.push_str("</div>");
    Html(html)
}
