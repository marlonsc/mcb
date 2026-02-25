use std::path::{Path, PathBuf};

use tree_sitter::{Language, Query, QueryCursor, StreamingIterator};

use crate::rules::yaml_loader::ValidatedRule;
use crate::{Callback, LANG, ParserTrait, ValidationError, action, guess_language};

#[derive(Debug, Clone)]
pub struct TreeSitterQueryMatch {
    pub file_path: PathBuf,
    pub line: usize,
    pub node_kind: String,
    pub capture_name: String,
}

#[derive(Debug)]
struct QueryExecutionCfg {
    query: String,
}

struct QueryExecutionCallback;

impl Callback for QueryExecutionCallback {
    type Res = crate::Result<Vec<TreeSitterQueryMatch>>;
    type Cfg = QueryExecutionCfg;

    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        let language = map_language(parser.get_language()).ok_or_else(|| {
            ValidationError::Config(format!(
                "Unsupported language for tree-sitter query execution: {:?}",
                parser.get_language()
            ))
        })?;

        let query = Query::new(&language, &cfg.query).map_err(|e| {
            ValidationError::Config(format!("Invalid tree-sitter query '{}': {e}", cfg.query))
        })?;

        let mut cursor = QueryCursor::new();
        let mut matches = Vec::new();
        let root = parser.get_root();
        let capture_names = query.capture_names();

        let mut query_matches = cursor.matches(&query, root.0, parser.get_code());
        while let Some(query_match) = query_matches.next() {
            for capture in query_match.captures {
                let start = capture.node.start_position();
                let capture_name = capture_names
                    .get(capture.index as usize)
                    .cloned()
                    .unwrap_or("unknown")
                    .to_owned();

                matches.push(TreeSitterQueryMatch {
                    file_path: PathBuf::new(),
                    line: start.row + 1,
                    node_kind: capture.node.kind().to_owned(),
                    capture_name,
                });
            }
        }

        Ok(matches)
    }
}

pub struct TreeSitterQueryExecutor;

impl TreeSitterQueryExecutor {
    pub fn execute(rule: &ValidatedRule, file: &Path) -> crate::Result<Vec<TreeSitterQueryMatch>> {
        let Some(query) = &rule.ast_query else {
            return Ok(Vec::new());
        };

        let source = std::fs::read(file).map_err(ValidationError::Io)?;
        let (lang, _) = guess_language(&source, file);
        let Some(lang) = lang else {
            return Ok(Vec::new());
        };

        let mut matches = action::<QueryExecutionCallback>(
            &lang,
            source,
            file,
            None,
            QueryExecutionCfg {
                query: query.clone(),
            },
        )?;

        for item in &mut matches {
            item.file_path = file.to_path_buf();
        }

        Ok(matches)
    }
}

fn map_language(lang: LANG) -> Option<Language> {
    match lang {
        LANG::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        LANG::Python => Some(tree_sitter_python::LANGUAGE.into()),
        LANG::Javascript | LANG::Mozjs => Some(tree_sitter_javascript::LANGUAGE.into()),
        LANG::Typescript | LANG::Tsx => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        LANG::Java => Some(tree_sitter_java::LANGUAGE.into()),
        LANG::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
        _ => None,
    }
}
