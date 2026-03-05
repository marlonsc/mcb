use mcb_utils::constants::validate::{CFG_TEST_MARKER, COMMENT_PREFIX};

pub(crate) fn for_each_non_test_non_comment_line<F>(content: &str, mut visit: F)
where
    F: FnMut(usize, &str, &str),
{
    let mut in_test_module = false;
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }
        if trimmed.contains(CFG_TEST_MARKER) {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        visit(line_num, line, trimmed);
    }
}

pub(crate) fn should_skip_crate(src_dir: &std::path::Path, excluded_crates: &[String]) -> bool {
    let Some(path_str) = src_dir.to_str() else {
        return false;
    };
    excluded_crates
        .iter()
        .any(|excluded| path_str.contains(excluded))
}
