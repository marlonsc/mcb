#[cfg(test)]
mod tests {
    use mcb_domain::ports::HighlightServiceInterface;
    use mcb_domain::test_utils::TestResult;
    use rstest::{fixture, rstest};

    #[fixture]
    fn highlight_service() -> TestResult<std::sync::Arc<dyn HighlightServiceInterface>> {
        mcb_domain::registry::services::resolve_highlight_service(&()).map_err(Into::into)
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_rust_code(
        highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    ) -> TestResult {
        let service = highlight_service?;
        let code = "fn main() { println!(\"Hello\"); }";

        let highlighted = service.highlight(code, "rust").await?;
        assert_eq!(highlighted.original, code);
        assert_eq!(highlighted.language, "rust");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Rust code"
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_empty_code(
        highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    ) -> TestResult {
        let service = highlight_service?;

        let highlighted = service.highlight("", "rust").await?;
        assert_eq!(highlighted.original, "");
        assert!(highlighted.spans.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_unsupported_language(
        highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    ) -> TestResult {
        let service = highlight_service?;

        let result = service.highlight("some code", "unsupported_lang").await;
        assert!(result.is_err(), "Should fail for unsupported language");
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_python_code(
        highlight_service: TestResult<std::sync::Arc<dyn HighlightServiceInterface>>,
    ) -> TestResult {
        let service = highlight_service?;
        let code = "def hello():\n    print('world')";

        let highlighted = service.highlight(code, "python").await?;
        assert_eq!(highlighted.language, "python");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Python code"
        );
        Ok(())
    }
}
