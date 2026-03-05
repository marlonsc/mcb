#[cfg(test)]
mod rmcp_http_feature_tests {
    use rstest::rstest;

    #[rstest]
    #[test]
    fn test_streamable_http_service_available() {
        use rmcp::transport::StreamableHttpServerConfig;
        use rmcp::transport::StreamableHttpService;

        let _ = std::any::type_name::<StreamableHttpServerConfig>();
        let _ = std::any::type_name::<StreamableHttpService<()>>();
    }
}
