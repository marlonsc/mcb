//! Filter pipeline logic for in-memory filtering.

use std::collections::HashSet;

use serde_json::Value;

use super::filter::{
    FilterParams, FilteredResult, SortOrder, parse_iso_date_to_epoch, parse_iso_date_to_epoch_end,
};

fn json_sort_key(v: &Value) -> String {
    match v {
        Value::String(s) => format!("s{}", s.to_lowercase()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                // Shift by i64::MIN so negative values sort correctly with zero-padding.
                format!("n{:020}", (i as u64).wrapping_add(i64::MIN as u64))
            } else if let Some(u) = n.as_u64() {
                format!("n{:020}", u.wrapping_add(i64::MIN as u64))
            } else {
                format!("n{:+021.6e}", n.as_f64().unwrap_or(0.0))
            }
        }
        Value::Bool(b) => format!("b{b}"),
        Value::Null => String::new(),
        other => format!("x{other}"),
    }
}

fn apply_search_filter(records: &mut Vec<Value>, search: Option<&str>) {
    let Some(query) = search.filter(|value| !value.is_empty()) else {
        return;
    };

    let query_lower = query.to_lowercase();
    records.retain(|rec| {
        if let Value::Object(map) = rec {
            map.values().any(|v| match v {
                Value::String(s) => s.to_lowercase().contains(&query_lower),
                _ => v.to_string().to_lowercase().contains(&query_lower),
            })
        } else {
            false
        }
    });
}

fn parse_date_range(params: &FilterParams) -> (Option<i64>, Option<i64>) {
    let epoch_from = params.date_from.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            parse_iso_date_to_epoch(s)
        }
    });
    let epoch_to = params.date_to.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            parse_iso_date_to_epoch_end(s)
        }
    });
    (epoch_from, epoch_to)
}

fn record_matches_date_range(rec: &Value, epoch_from: Option<i64>, epoch_to: Option<i64>) -> bool {
    if let Value::Object(map) = rec {
        let mut has_any_ts = false;
        let in_range = map
            .iter()
            .filter(|(k, _)| k.ends_with("_at"))
            .filter_map(|(_, v)| match v {
                Value::Number(n) => n.as_i64(),
                _ => None,
            })
            .any(|ts| {
                has_any_ts = true;
                let after = epoch_from.is_none_or(|from| ts >= from);
                let before = epoch_to.is_none_or(|to| ts <= to);
                after && before
            });
        in_range || !has_any_ts
    } else {
        true
    }
}

fn apply_date_filter(records: &mut Vec<Value>, epoch_from: Option<i64>, epoch_to: Option<i64>) {
    if epoch_from.is_none() && epoch_to.is_none() {
        return;
    }
    records.retain(|rec| record_matches_date_range(rec, epoch_from, epoch_to));
}

fn apply_sort(records: &mut [Value], params: &FilterParams, valid_sort_fields: &HashSet<String>) {
    let Some(field) = params.sort_field.as_ref() else {
        return;
    };
    if !valid_sort_fields.contains(field.as_str()) {
        return;
    }

    let desc = matches!(params.sort_order, Some(SortOrder::Desc));
    records.sort_by(|a, b| {
        let va = a.get(field).map(json_sort_key);
        let vb = b.get(field).map(json_sort_key);
        let cmp = va.cmp(&vb);
        if desc { cmp.reverse() } else { cmp }
    });
}

fn paginate(records: Vec<Value>, params: &FilterParams) -> FilteredResult {
    let total_count = records.len();
    let per_page = if params.per_page == 0 {
        20
    } else {
        params.per_page
    };
    let total_pages = if total_count == 0 {
        0
    } else {
        total_count.div_ceil(per_page)
    };
    let page = if params.page == 0 { 1 } else { params.page };
    let start = (page - 1) * per_page;
    let page_records = if start >= total_count {
        Vec::new()
    } else {
        let end = (start + per_page).min(total_count);
        records[start..end].to_vec()
    };

    FilteredResult {
        records: page_records,
        total_count,
        page,
        per_page,
        total_pages,
    }
}

/// Apply in-memory filtering, sorting, and pagination to a pre-fetched record list.
#[must_use]
pub fn apply_filter_pipeline(
    mut records: Vec<Value>,
    params: &FilterParams,
    valid_sort_fields: &HashSet<String>,
) -> FilteredResult {
    apply_search_filter(&mut records, params.search.as_deref());
    let (epoch_from, epoch_to) = parse_date_range(params);
    apply_date_filter(&mut records, epoch_from, epoch_to);
    apply_sort(&mut records, params, valid_sort_fields);
    paginate(records, params)
}
