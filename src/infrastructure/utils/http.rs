//! HTTP Response utilities - Replaces ~8 lines per call across embedding providers (DRY)
//!
//! Consolidates the common pattern of checking response status and extracting errors

use crate::domain::error::{Error, Result};
use axum::http::StatusCode;

/// Helper trait for converting Results to HTTP StatusCode errors
///
/// Replaces `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` pattern
pub trait IntoStatusCode<T> {
    /// Convert to StatusCode::INTERNAL_SERVER_ERROR on error
    fn to_500(self) -> std::result::Result<T, StatusCode>;

    /// Convert to StatusCode::NOT_FOUND on error
    fn to_404(self) -> std::result::Result<T, StatusCode>;

    /// Convert to StatusCode::BAD_REQUEST on error
    fn to_400(self) -> std::result::Result<T, StatusCode>;
}

impl<T, E> IntoStatusCode<T> for std::result::Result<T, E> {
    #[inline]
    fn to_500(self) -> std::result::Result<T, StatusCode> {
        self.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    #[inline]
    fn to_404(self) -> std::result::Result<T, StatusCode> {
        self.map_err(|_| StatusCode::NOT_FOUND)
    }

    #[inline]
    fn to_400(self) -> std::result::Result<T, StatusCode> {
        self.map_err(|_| StatusCode::BAD_REQUEST)
    }
}

/// HTTP response utilities for API clients (embedding providers, etc.)
///
/// Consolidates the common pattern of checking response status and extracting errors.
pub struct HttpResponseUtils;

impl HttpResponseUtils {
    /// Check HTTP response and return error if not successful (saves ~8 lines per use)
    ///
    /// # Example
    /// ```ignore
    /// let response = client.post(url).send().await?;
    /// HttpResponseUtils::check_response(response, "OpenAI").await?;
    /// // vs. the old 8-line pattern:
    /// // if !response.status().is_success() {
    /// //     let status = response.status();
    /// //     let error_text = response.text().await.unwrap_or_default();
    /// //     return Err(Error::embedding(format!("API error {}: {}", status, error_text)));
    /// // }
    /// ```
    pub async fn check_response(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<reqwest::Response> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::embedding(format!(
                "{} API error {}: {}",
                provider_name, status, error_text
            )));
        }
        Ok(response)
    }

    /// Parse JSON from response with provider-specific error (saves ~6 lines per use)
    pub async fn json_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<T> {
        response
            .json()
            .await
            .map_err(|e| Error::embedding(format!("{} response parse error: {}", provider_name, e)))
    }

    /// Combined: check response status and parse JSON (saves ~12 lines per use)
    pub async fn check_and_parse<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<T> {
        let response = Self::check_response(response, provider_name).await?;
        Self::json_response(response, provider_name).await
    }
}
