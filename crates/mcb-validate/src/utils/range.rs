//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Range and interval utilities.
//!
//! Generic helpers for line ranges, intervals, and overlap checks used by
//! duplication detection and other validators.

/// Returns whether two inclusive line ranges `[start1..=end1]` and `[start2..=end2]` overlap.
///
/// Ranges overlap when they share at least one line (neither ends before the other starts).
#[must_use]
pub fn lines_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
    !(end1 < start2 || end2 < start1)
}
