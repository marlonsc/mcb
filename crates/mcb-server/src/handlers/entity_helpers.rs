use mcb_domain::error::Error;
use mcb_domain::value_objects::OrgContext;
use rmcp::model::ErrorData as McpError;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) fn current_org_id() -> String {
    OrgContext::current().id_str().to_string()
}

pub(crate) fn require_data<T>(
    data: Option<Value>,
    required_message: &'static str,
) -> Result<T, McpError>
where
    T: DeserializeOwned,
{
    let data = data.ok_or_else(|| McpError::invalid_params(required_message, None))?;
    serde_json::from_value(data).map_err(|_| McpError::invalid_params("invalid data", None))
}

pub(crate) fn map_opaque_error<T>(result: Result<T, Error>) -> Result<T, McpError> {
    result.map_err(crate::error_mapping::to_opaque_mcp_error)
}
