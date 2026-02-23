//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
use crate::validators::solid::constants::{
    MAX_AFFIX_LENGTH, MIN_AFFIX_LENGTH, MIN_NAMES_FOR_RELATION_CHECK,
    MIN_WORD_LENGTH_FOR_COMPARISON,
};

pub(super) fn structs_seem_related_impl(names: &[String]) -> bool {
    if names.len() < MIN_NAMES_FOR_RELATION_CHECK {
        return true;
    }

    let checks = [
        has_common_prefix,
        has_common_suffix,
        has_purpose_suffix,
        has_shared_keyword,
        has_common_words,
    ];

    checks.iter().any(|check| check(names))
}

fn has_common_prefix(names: &[String]) -> bool {
    let first = &names[0];
    for len in (MIN_AFFIX_LENGTH..=first.len().min(MAX_AFFIX_LENGTH)).rev() {
        let prefix = &first[..len];
        if names.iter().all(|n| n.starts_with(prefix)) {
            return true;
        }
    }
    false
}

fn has_common_suffix(names: &[String]) -> bool {
    let first = &names[0];
    for len in (MIN_AFFIX_LENGTH..=first.len().min(MAX_AFFIX_LENGTH)).rev() {
        let suffix = &first[first.len().saturating_sub(len)..];
        if names.iter().all(|n| n.ends_with(suffix)) {
            return true;
        }
    }
    false
}

fn has_purpose_suffix(names: &[String]) -> bool {
    const PURPOSE_SUFFIXES: [&str; 21] = [
        "Config",
        "State",
        "Error",
        "Request",
        "Response",
        "Options",
        "Args",
        "Report",
        "Entry",
        "Info",
        "Data",
        "Metrics",
        "Operation",
        "Status",
        "Result",
        "Summary",
        "File",
        "Match",
        "Check",
        "Health",
        "Complexity",
    ];
    names
        .iter()
        .any(|n| PURPOSE_SUFFIXES.iter().any(|suffix| n.ends_with(suffix)))
}

fn has_shared_keyword(names: &[String]) -> bool {
    const DOMAIN_KEYWORDS: [&str; 44] = [
        "Config",
        "Options",
        "Settings",
        "Error",
        "Result",
        "Builder",
        "Handler",
        "Provider",
        "Service",
        "Health",
        "Crypto",
        "Admin",
        "Http",
        "Args",
        "Request",
        "Response",
        "State",
        "Status",
        "Info",
        "Data",
        "Message",
        "Event",
        "Token",
        "Auth",
        "Cache",
        "Index",
        "Search",
        "Chunk",
        "Embed",
        "Vector",
        "Transport",
        "Operation",
        "Mcp",
        "Protocol",
        "Server",
        "Client",
        "Connection",
        "Session",
        "Route",
        "Endpoint",
        "Memory",
        "Observation",
        "Filter",
        "Pattern",
    ];

    DOMAIN_KEYWORDS.iter().any(|keyword| {
        let has_keyword: Vec<_> = names.iter().filter(|n| n.contains(keyword)).collect();
        has_keyword.len() > names.len() / 2
    })
}

fn has_common_words(names: &[String]) -> bool {
    let words: Vec<Vec<&str>> = names
        .iter()
        .map(|n| crate::utils::naming::split_camel_case(n))
        .collect();

    if let Some(first_words) = words.first() {
        for word in first_words {
            if word.len() >= MIN_WORD_LENGTH_FOR_COMPARISON {
                let count = words.iter().filter(|w| w.contains(word)).count();
                if count > names.len() / 2 {
                    return true;
                }
            }
        }
    }
    false
}
