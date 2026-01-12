//! Handler for the search_code MCP tool
//!
//! This handler is responsible for performing semantic code search.
//! It validates queries, checks permissions, manages caching, and coordinates
//! the search process with proper error handling and timeouts.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use serde_json;
use std::sync::Arc;
use std::time::Instant;

use crate::application::SearchService;
use crate::domain::validation::{StringValidator, StringValidatorTrait, ValidationError};
use crate::infrastructure::auth::Permission;
use crate::infrastructure::cache::{CacheManager, CacheResult};
use crate::infrastructure::limits::ResourceLimits;
use crate::server::args::SearchCodeArgs;
use crate::server::auth::AuthHandler;
use crate::server::formatter::ResponseFormatter;

/// Handler for code search operations
pub struct SearchCodeHandler {
    search_service: Arc<SearchService>,
    auth_handler: Arc<AuthHandler>,
    resource_limits: Arc<ResourceLimits>,
    cache_manager: Arc<CacheManager>,
}

impl SearchCodeHandler {
    /// Create a new search_code handler
    pub fn new(
        search_service: Arc<SearchService>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        Self {
            search_service,
            auth_handler,
            resource_limits,
            cache_manager,
        }
    }

    /// Validate search query using the validation framework
    fn validate_search_query(&self, query: &str) -> Result<String, CallToolResult> {
        let trimmed = query.trim();

        // Create validator for search queries
        let validator = StringValidator::not_empty()
            .combine_with(StringValidator::min_length(3))
            .combine_with(StringValidator::max_length(1000)); // Reasonable limit

        match validator.validate(trimmed) {
            Ok(validated) => Ok(validated),
            Err(ValidationError::Required { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    "Search query cannot be empty. Please provide a natural language query.",
                ))
            }
            Err(ValidationError::TooShort { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    "Search query too short. Please use at least 3 characters for meaningful results.",
                ))
            }
            Err(ValidationError::TooLong { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    "Search query too long. Please limit to 1000 characters.",
                ))
            }
            _ => Err(ResponseFormatter::format_query_validation_error(
                "Invalid search query format.",
            )),
        }
    }

    /// Handle the search_code tool request
    pub async fn handle(
        &self,
        Parameters(SearchCodeArgs {
            query,
            limit,
            token,
            filters: _,
            ..
        }): Parameters<SearchCodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        // Check authentication and permissions
        if let Err(e) = self
            .auth_handler
            .check_auth(token.as_ref(), &Permission::SearchCodebase)
        {
            return Ok(ResponseFormatter::format_auth_error(&e.to_string()));
        }

        // Check resource limits for search operation
        if let Err(e) = self.resource_limits.check_operation_allowed("search").await {
            return Ok(ResponseFormatter::format_resource_limit_error(
                &e.to_string(),
            ));
        }

        // Acquire search permit
        let _permit = match self
            .resource_limits
            .acquire_operation_permit("search")
            .await
        {
            Ok(permit) => permit,
            Err(e) => {
                return Ok(ResponseFormatter::format_resource_limit_error(
                    &e.to_string(),
                ));
            }
        };

        // Validate query input using validation framework
        let query = match self.validate_search_query(&query) {
            Ok(validated) => validated,
            Err(error_response) => return Ok(error_response),
        };

        // Validate limit
        let limit = limit.clamp(1, 50); // Reasonable bounds for performance
        let collection = "default";

        // Check cache for search results
        let cache_key = format!("{}:{}:{}", collection, query, limit);
        let cached_result: CacheResult<serde_json::Value> =
            self.cache_manager.get("search_results", &cache_key).await;

        if let CacheResult::Hit(cached_data) = cached_result {
            if let Ok(search_results) =
                serde_json::from_value::<Vec<crate::domain::types::SearchResult>>(cached_data)
            {
                tracing::info!(
                    "✅ Search cache hit for query: '{}' (limit: {})",
                    query,
                    limit
                );
                return ResponseFormatter::format_search_response(
                    &query,
                    &search_results,
                    start_time.elapsed(),
                    true,
                );
            }
        }

        tracing::info!(
            "Performing semantic search for query: '{}' (limit: {})",
            query,
            limit
        );

        // Add timeout for search operations
        let search_future = self.search_service.search(collection, &query, limit);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second timeout
            search_future,
        )
        .await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(results)) => {
                // Cache search results
                let _ = self
                    .cache_manager
                    .set(
                        "search_results",
                        &cache_key,
                        serde_json::to_value(&results).unwrap_or_default(),
                    )
                    .await;

                // Use the response formatter
                ResponseFormatter::format_search_response(&query, &results, duration, false)
            }
            Ok(Err(e)) => Ok(ResponseFormatter::format_search_error(
                &e.to_string(),
                &query,
            )),
            Err(_) => Ok(ResponseFormatter::format_search_timeout(&query)),
        }
    }

}
