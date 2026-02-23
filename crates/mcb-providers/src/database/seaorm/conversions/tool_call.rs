//! ToolCall domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::tool_call;
use mcb_domain::entities::ToolCall;

impl From<tool_call::Model> for ToolCall {
    fn from(m: tool_call::Model) -> Self {
        Self {
            id: m.id,
            session_id: m.session_id,
            tool_name: m.tool_name,
            params_summary: m.params_summary,
            success: m.success != 0,
            error_message: m.error_message,
            duration_ms: m.duration_ms,
            created_at: m.created_at,
        }
    }
}

impl From<ToolCall> for tool_call::ActiveModel {
    fn from(e: ToolCall) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::NotSet,
            project_id: ActiveValue::NotSet,
            repo_id: ActiveValue::NotSet,
            session_id: ActiveValue::Set(e.session_id),
            tool_name: ActiveValue::Set(e.tool_name),
            params_summary: ActiveValue::Set(e.params_summary),
            success: ActiveValue::Set(i64::from(e.success)),
            error_message: ActiveValue::Set(e.error_message),
            duration_ms: ActiveValue::Set(e.duration_ms),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool_call() -> ToolCall {
        ToolCall {
            id: "tc-001".into(),
            session_id: "ses-001".into(),
            tool_name: "mcp_read".into(),
            params_summary: Some("file=/src/main.rs".into()),
            success: true,
            error_message: None,
            duration_ms: Some(150),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_tool_call() {
        let domain = sample_tool_call();
        let active: tool_call::ActiveModel = domain.clone().into();

        let model = tool_call::Model {
            id: active.id.unwrap(),
            org_id: None,
            project_id: None,
            repo_id: None,
            session_id: active.session_id.unwrap(),
            tool_name: active.tool_name.unwrap(),
            params_summary: active.params_summary.unwrap(),
            success: active.success.unwrap(),
            error_message: active.error_message.unwrap(),
            duration_ms: active.duration_ms.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: ToolCall = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.success, true);
        assert_eq!(back.tool_name, "mcp_read");
    }
}
