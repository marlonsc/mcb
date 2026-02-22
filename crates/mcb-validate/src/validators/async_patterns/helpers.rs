use regex::Regex;

fn brace_delta(line: &str) -> i32 {
    let open = line.chars().filter(|c| *c == '{').count();
    let close = line.chars().filter(|c| *c == '}').count();
    i32::try_from(open).unwrap_or(i32::MAX) - i32::try_from(close).unwrap_or(i32::MAX)
}

pub(crate) fn for_each_async_fn_line<F>(content: &str, async_fn_pattern: &Regex, mut visit: F)
where
    F: FnMut(usize, &str, &str),
{
    let mut in_async_fn = false;
    let mut async_fn_depth = 0;

    crate::validators::for_each_non_test_non_comment_line(content, |line_num, line, trimmed| {
        if async_fn_pattern.is_match(trimmed) {
            in_async_fn = true;
            async_fn_depth = 0;
        }

        if in_async_fn {
            async_fn_depth += brace_delta(line);
            visit(line_num, line, trimmed);
            if async_fn_depth <= 0 {
                in_async_fn = false;
            }
        }
    });
}
