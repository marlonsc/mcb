//! Tests for FileTreeNode traversal and formatting methods
//!
//! Covers: traverse, Display, to_ansi, to_html methods

use mcb_domain::value_objects::FileTreeNode;

#[test]
fn test_traverse_visits_all_nodes() {
    let mut root = FileTreeNode::directory("src", "src");
    root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));
    root.add_child(FileTreeNode::file("main.rs", "src/main.rs", 5, "rust"));

    let mut visited = Vec::new();
    root.traverse(&mut |node| {
        visited.push(node.name.clone());
    });

    assert_eq!(visited.len(), 3);
    assert_eq!(visited[0], "src");
    assert!(visited.contains(&"lib.rs".to_string()));
    assert!(visited.contains(&"main.rs".to_string()));
}

#[test]
fn test_traverse_depth_first_order() {
    let mut root = FileTreeNode::directory("root", "root");
    let mut subdir = FileTreeNode::directory("subdir", "root/subdir");
    subdir.add_child(FileTreeNode::file(
        "file.rs",
        "root/subdir/file.rs",
        3,
        "rust",
    ));
    root.add_child(subdir);

    let mut visited = Vec::new();
    root.traverse(&mut |node| {
        visited.push(node.name.clone());
    });

    assert_eq!(visited.len(), 3);
    assert_eq!(visited[0], "root");
    assert_eq!(visited[1], "subdir");
    assert_eq!(visited[2], "file.rs");
}

#[test]
fn test_display_trait_formats_tree() {
    let mut root = FileTreeNode::directory("src", "src");
    root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));

    let display_output = format!("{}", root);

    assert!(display_output.contains("src"));
    assert!(display_output.contains("lib.rs"));
    assert!(!display_output.is_empty());
}

#[test]
fn test_to_ansi_contains_tree_structure() {
    let mut root = FileTreeNode::directory("src", "src");
    root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));

    let ansi = root.to_ansi();

    assert!(ansi.contains("src"));
    assert!(ansi.contains("lib.rs"));
    assert!(ansi.contains("└──") || ansi.contains("├──"));
}

#[test]
fn test_to_ansi_includes_chunk_count() {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", 42, "rust");
    let ansi = file.to_ansi();

    assert!(ansi.contains("lib.rs"));
    assert!(ansi.contains("42"));
}

#[test]
fn test_to_html_valid_structure() {
    let mut root = FileTreeNode::directory("src", "src");
    root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));

    let html = root.to_html();

    assert!(html.contains("<ul>"));
    assert!(html.contains("</ul>"));
    assert!(html.contains("<li>"));
    assert!(html.contains("</li>"));
    assert!(html.contains("src"));
    assert!(html.contains("lib.rs"));
}

#[test]
fn test_to_html_includes_icons() {
    let mut root = FileTreeNode::directory("src", "src");
    root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));

    let html = root.to_html();

    assert!(html.contains("📁")); // folder icon
    assert!(html.contains("📄")); // file icon
}

#[test]
fn test_to_html_escapes_special_characters() {
    let mut root = FileTreeNode::directory("src<script>", "src");
    root.add_child(FileTreeNode::file(
        "lib&test.rs",
        "src/lib&test.rs",
        10,
        "rust",
    ));

    let html = root.to_html();

    assert!(html.contains("&lt;"));
    assert!(html.contains("&gt;"));
    assert!(html.contains("&amp;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn test_to_html_includes_chunk_count() {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", 42, "rust");
    let html = file.to_html();

    assert!(html.contains("42"));
}

#[test]
fn test_traverse_empty_tree() {
    let root = FileTreeNode::directory("empty", "empty");

    let mut count = 0;
    root.traverse(&mut |_node| {
        count += 1;
    });

    assert_eq!(count, 1); // just the root
}

#[test]
fn test_to_ansi_nested_structure() {
    let mut root = FileTreeNode::directory("root", "root");
    let mut level1 = FileTreeNode::directory("level1", "root/level1");
    let mut level2 = FileTreeNode::directory("level2", "root/level1/level2");
    level2.add_child(FileTreeNode::file(
        "deep.rs",
        "root/level1/level2/deep.rs",
        5,
        "rust",
    ));
    level1.add_child(level2);
    root.add_child(level1);

    let ansi = root.to_ansi();

    assert!(ansi.contains("root"));
    assert!(ansi.contains("level1"));
    assert!(ansi.contains("level2"));
    assert!(ansi.contains("deep.rs"));
}

#[test]
fn test_to_html_nested_structure() {
    let mut root = FileTreeNode::directory("root", "root");
    let mut level1 = FileTreeNode::directory("level1", "root/level1");
    let mut level2 = FileTreeNode::directory("level2", "root/level1/level2");
    level2.add_child(FileTreeNode::file(
        "deep.rs",
        "root/level1/level2/deep.rs",
        5,
        "rust",
    ));
    level1.add_child(level2);
    root.add_child(level1);

    let html = root.to_html();

    assert!(html.contains("root"));
    assert!(html.contains("level1"));
    assert!(html.contains("level2"));
    assert!(html.contains("deep.rs"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("</ul>"));
}

#[test]
fn test_traverse_callback_receives_correct_nodes() {
    let mut root = FileTreeNode::directory("src", "src");
    let file1 = FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust");
    let file2 = FileTreeNode::file("main.rs", "src/main.rs", 5, "rust");
    root.add_child(file1);
    root.add_child(file2);

    let mut paths = Vec::new();
    root.traverse(&mut |node| {
        paths.push(node.path.clone());
    });

    assert_eq!(paths.len(), 3);
    assert!(paths.contains(&"src".to_string()));
    assert!(paths.contains(&"src/lib.rs".to_string()));
    assert!(paths.contains(&"src/main.rs".to_string()));
}

#[test]
fn test_to_ansi_single_file() {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust");
    let ansi = file.to_ansi();

    assert!(ansi.contains("lib.rs"));
    assert!(ansi.contains("10"));
}

#[test]
fn test_to_html_single_file() {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust");
    let html = file.to_html();

    assert!(html.contains("lib.rs"));
    assert!(html.contains("10"));
    assert!(html.contains("📄"));
}
