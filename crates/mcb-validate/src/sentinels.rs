//! Provenance sentinels for validator report fields.
//!
//! These are display-only markers for report fields whose true value could
//! not be determined (an unnamed capture, a dependency without a source, a
//! path-less violation). They are deliberately crate-local: mcb-utils' generic
//! `FALLBACK_UNKNOWN` was a cross-domain fallback constant, which the
//! repository law forbids; each domain now names its own sentinel. The module
//! is named `sentinels` because that is what it is — CA016 forbids generic
//! per-crate `constants` modules; the SSOT for shared constants is
//! `mcb_utils::constants`.

/// Report-field sentinel: the validator could not determine the provenance
/// (file, item, or dependency name) for this entry.
pub const UNKNOWN_PROVENANCE: &str = "unknown";
