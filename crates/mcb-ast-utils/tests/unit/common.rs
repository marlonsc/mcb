use tree_sitter::Parser;

pub fn parse_rust(code: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");
    parser.parse(code, None).unwrap()
}
