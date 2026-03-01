#[cfg(test)]
mod tests {
    use mcb_domain::ports::HighlightServiceInterface;
    use rstest::rstest;

    fn highlight_service() -> std::sync::Arc<dyn HighlightServiceInterface> {
        mcb_domain::registry::services::resolve_highlight_service(&())
            .expect("highlight service should resolve")
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_rust_code() {
        let service = highlight_service();
        let code = "fn main() { println!(\"Hello\"); }";

        let result = service.highlight(code, "rust").await;

        let highlighted = result.expect("highlight Rust code should succeed");
        assert_eq!(highlighted.original, code);
        assert_eq!(highlighted.language, "rust");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Rust code"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_empty_code() {
        let service = highlight_service();
        let code = "";

        let result = service.highlight(code, "rust").await;

        let highlighted = result.expect("highlight empty code should succeed");
        assert_eq!(highlighted.original, "");
        assert!(highlighted.spans.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_unsupported_language() {
        let service = highlight_service();
        let code = "some code";

        let result = service.highlight(code, "unsupported_lang").await;

        assert!(result.is_err(), "Should fail for unsupported language");
    }

    #[rstest]
    #[tokio::test]
    async fn test_highlight_python_code() {
        let service = highlight_service();
        let code = "def hello():\n    print('world')";

        let result = service.highlight(code, "python").await;

        let highlighted = result.expect("highlight Python code should succeed");
        assert_eq!(highlighted.language, "python");
        assert!(
            !highlighted.spans.is_empty(),
            "Should have highlight spans for Python code"
        );
    }
}
