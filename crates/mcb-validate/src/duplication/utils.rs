//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
#[must_use]
pub(crate) fn lines_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
    !(end1 < start2 || end2 < start1)
}
