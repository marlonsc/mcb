use std::fmt;

use serde::{Deserialize, Serialize};

/// Tree node for hierarchical file navigation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTreeNode {
    /// Display name of the node (file or directory name)
    pub name: String,
    /// Full path from repository root
    pub path: String,
    /// Whether this node is a directory
    pub is_dir: bool,
    /// Child nodes (empty for files)
    pub children: Vec<FileTreeNode>,
    /// Number of chunks (only for files)
    pub chunk_count: Option<u32>,
    /// Detected language (only for files)
    pub language: Option<String>,
}

impl FileTreeNode {
    /// Create a new file node
    pub fn file(
        name: impl Into<String>,
        path: impl Into<String>,
        chunk_count: u32,
        language: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: false,
            children: Vec::new(),
            chunk_count: Some(chunk_count),
            language: Some(language.into()),
        }
    }

    /// Create a new directory node
    pub fn directory(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: true,
            children: Vec::new(),
            chunk_count: None,
            language: None,
        }
    }

    /// Add a child node to this directory (builder pattern)
    pub fn with_child(mut self, child: FileTreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Sort children: directories first, then files, alphabetically
    #[must_use]
    pub fn sorted(mut self) -> Self {
        self.sort_children();
        self
    }

    /// Sort children: directories first, then files, alphabetically (in-place)
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }

    /// Traverse the tree and call a callback for each node
    pub fn traverse(&self, callback: &mut dyn FnMut(&FileTreeNode)) {
        callback(self);
        for child in &self.children {
            child.traverse(callback);
        }
    }

    /// Convert tree to ANSI-formatted string with colors and tree structure
    pub fn to_ansi(&self) -> String {
        let mut output = String::new();
        self.format_ansi(&mut output, "", true);
        output
    }

    fn format_ansi(&self, output: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let color = if self.is_dir { "\x1b[34m" } else { "\x1b[0m" };
        let reset = "\x1b[0m";

        output.push_str(prefix);
        output.push_str(connector);
        output.push_str(color);
        output.push_str(&self.name);
        output.push_str(reset);

        if let Some(count) = self.chunk_count {
            output.push_str(&format!(" ({})", count));
        }
        output.push('\n');

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        for (i, child) in self.children.iter().enumerate() {
            let is_last_child = i == self.children.len() - 1;
            child.format_ansi(output, &new_prefix, is_last_child);
        }
    }

    /// Convert tree to HTML-formatted string with nesting
    pub fn to_html(&self) -> String {
        let mut output = String::new();
        self.format_html(&mut output);
        output
    }

    fn format_html(&self, output: &mut String) {
        let icon = if self.is_dir { "📁" } else { "📄" };
        let name_html = html_escape(&self.name);

        output.push_str("<ul>\n");
        output.push_str("<li>");
        output.push_str(icon);
        output.push(' ');
        output.push_str(&name_html);

        if let Some(count) = self.chunk_count {
            output.push_str(&format!(" <span style=\"color: #888;\">({})</span>", count));
        }

        if !self.children.is_empty() {
            output.push('\n');
            for child in &self.children {
                child.format_html(output);
            }
            output.push_str("</li>\n");
            output.push_str("</ul>\n");
        } else {
            output.push_str("</li>\n");
            output.push_str("</ul>\n");
        }
    }
}

/// HTML escape a string to prevent XSS
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl fmt::Display for FileTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}
