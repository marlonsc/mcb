//! Tests for FileTreeNode traversal and formatting methods
//!
//! Covers: traverse, Display, to_ansi, to_html methods

use mcb_domain::value_objects::FileTreeNode;
use rstest::*;

#[rstest]
#[case(false)]
#[case(true)]
fn traverse_variants(#[case] nested: bool) {
    let root = if nested {
        let root = FileTreeNode::directory("root", "root");
        let subdir = FileTreeNode::directory("subdir", "root/subdir");
        let subdir = subdir.with_child(FileTreeNode::file(
            "file.rs",
            "root/subdir/file.rs",
            3,
            "rust",
        ));
        root.with_child(subdir)
    } else {
        let mut root = FileTreeNode::directory("src", "src");
        root = root.with_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));
        root.with_child(FileTreeNode::file("main.rs", "src/main.rs", 5, "rust"))
    };

    let mut visited = Vec::new();
    root.traverse(&mut |node| visited.push(node.name.clone()));

    assert_eq!(visited.len(), 3);
    if nested {
        assert_eq!(visited, vec!["root", "subdir", "file.rs"]);
    } else {
        assert_eq!(visited[0], "src");
        assert!(visited.contains(&"lib.rs".to_string()));
        assert!(visited.contains(&"main.rs".to_string()));
    }
}

#[test]
fn test_display_trait_formats_tree() {
    let mut root = FileTreeNode::directory("src", "src");
    root = root.with_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));

    let display_output = format!("{}", root);

    assert!(display_output.contains("src"));
    assert!(display_output.contains("lib.rs"));
    assert!(!display_output.is_empty());
}

#[rstest]
#[case(false, "lib.rs", 42)]
#[case(true, "lib.rs", 10)]
#[test]
fn test_to_ansi_basic_output(
    #[case] as_tree: bool,
    #[case] expected_name: &str,
    #[case] expected_chunks: u32,
) {
    let node = if as_tree {
        let mut root = FileTreeNode::directory("src", "src");
        root = root.with_child(FileTreeNode::file(
            expected_name,
            "src/lib.rs",
            expected_chunks,
            "rust",
        ));
        root
    } else {
        FileTreeNode::file(expected_name, "src/lib.rs", expected_chunks, "rust")
    };

    let ansi = node.to_ansi();
    assert!(ansi.contains(expected_name));
    assert!(ansi.contains(&expected_chunks.to_string()));
    if as_tree {
        assert!(ansi.contains("└──") || ansi.contains("├──"));
    }
}

#[rstest]
#[case("src", "lib.rs", 10)]
#[test]
fn test_to_html_valid_structure_and_icons(
    #[case] dir_name: &str,
    #[case] file_name: &str,
    #[case] chunk_count: u32,
) {
    let mut root = FileTreeNode::directory(dir_name, "src");
    root = root.with_child(FileTreeNode::file(
        file_name,
        "src/lib.rs",
        chunk_count,
        "rust",
    ));

    let html = root.to_html();

    assert!(html.contains("<ul>"));
    assert!(html.contains("</ul>"));
    assert!(html.contains("<li>"));
    assert!(html.contains("</li>"));
    assert!(html.contains(dir_name));
    assert!(html.contains(file_name));
    assert!(html.contains("📁"));
    assert!(html.contains("📄"));
}

#[test]
fn test_to_html_escapes_special_characters() {
    let mut root = FileTreeNode::directory("src<script>", "src");
    root = root.with_child(FileTreeNode::file(
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

#[rstest]
#[case(42)]
#[case(10)]
#[test]
fn test_to_html_includes_chunk_count(#[case] chunk_count: u32) {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", chunk_count, "rust");
    let html = file.to_html();

    assert!(html.contains(&chunk_count.to_string()));
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

#[rstest]
#[case("ansi")]
#[case("html")]
fn nested_structure_rendering(#[case] render_mode: &str) {
    let mut root = FileTreeNode::directory("root", "root");
    let mut level1 = FileTreeNode::directory("level1", "root/level1");
    let mut level2 = FileTreeNode::directory("level2", "root/level1/level2");
    level2 = level2.with_child(FileTreeNode::file(
        "deep.rs",
        "root/level1/level2/deep.rs",
        5,
        "rust",
    ));
    level1 = level1.with_child(level2);
    root = root.with_child(level1);

    let rendered = if render_mode == "ansi" {
        root.to_ansi()
    } else {
        root.to_html()
    };

    assert!(rendered.contains("root"));
    assert!(rendered.contains("level1"));
    assert!(rendered.contains("level2"));
    assert!(rendered.contains("deep.rs"));
    if render_mode == "html" {
        assert!(rendered.contains("<ul>"));
        assert!(rendered.contains("</ul>"));
    }
}

#[test]
fn test_traverse_callback_receives_correct_nodes() {
    let mut root = FileTreeNode::directory("src", "src");
    let file1 = FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust");
    let file2 = FileTreeNode::file("main.rs", "src/main.rs", 5, "rust");
    root = root.with_child(file1);
    root = root.with_child(file2);

    let mut paths = Vec::new();
    root.traverse(&mut |node| {
        paths.push(node.path.clone());
    });

    assert_eq!(paths.len(), 3);
    assert!(paths.contains(&"src".to_string()));
    assert!(paths.contains(&"src/lib.rs".to_string()));
    assert!(paths.contains(&"src/main.rs".to_string()));
}

#[rstest]
#[case("ansi")]
#[case("html")]
#[test]
fn test_single_file_rendering(#[case] render_mode: &str) {
    let file = FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust");
    let rendered = if render_mode == "ansi" {
        file.to_ansi()
    } else {
        file.to_html()
    };

    assert!(rendered.contains("lib.rs"));
    assert!(rendered.contains("10"));
    if render_mode == "html" {
        assert!(rendered.contains("📄"));
    }
}
