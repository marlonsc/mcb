#[cfg(test)]
mod tests {
    use mcb_domain::ports::browse::HighlightServiceInterface;
    use mcb_infrastructure::services::highlight_service::HighlightServiceImpl;

    #[tokio::test]
    async fn test_highlight_rust_code() {
        let service = HighlightServiceImpl::new();
        let code = "fn main() { println!(\"Hello\"); }";

        let result = service.highlight(code, "rust").await;

        assert!(result.is_ok());
        let highlighted = result.unwrap();
        assert_eq!(highlighted.original, code);
        assert_eq!(highlighted.language, "rust");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Rust code"
        );
    }

    #[tokio::test]
    async fn test_highlight_empty_code() {
        let service = HighlightServiceImpl::new();
        let code = "";

        let result = service.highlight(code, "rust").await;

        assert!(result.is_ok());
        let highlighted = result.unwrap();
        assert_eq!(highlighted.original, "");
        assert!(highlighted.spans.is_empty());
    }

    #[tokio::test]
    async fn test_highlight_unsupported_language() {
        let service = HighlightServiceImpl::new();
        let code = "some code";

        let result = service.highlight(code, "unsupported_lang").await;

        assert!(result.is_err(), "Should fail for unsupported language");
    }

    #[tokio::test]
    async fn test_highlight_python_code() {
        let service = HighlightServiceImpl::new();
        let code = "def hello():\n    print('world')";

        let result = service.highlight(code, "python").await;

        assert!(result.is_ok());
        let highlighted = result.unwrap();
        assert_eq!(highlighted.language, "python");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Python code"
        );
    }
}
