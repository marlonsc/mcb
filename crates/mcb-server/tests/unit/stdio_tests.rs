use mcb_server::McpServer;
use mcb_server::transport::stdio::StdioServerExt;

#[test]
fn test_stdio_server_ext_trait_exists() {
    // Basic compilation check that the trait is visible
    let _ = <McpServer as StdioServerExt>::serve_stdio;
}
