# Design

## Context

The test asserts `size_of_val(erased)` equals `2 * size_of::<usize>()`. `std::mem::size_of_val` returns the size of the value the reference points to (the underlying `Git2Provider`), not the size of the fat pointer itself. Hence the assertion is conceptually wrong and fails (0 vs 16). Object safety is already proven by `_assert_object_safe(&provider)` compiling.

## Goals / Non-Goals

**Goals:**

- Fix the failing test so CI passes
- Preserve object-safety verification via the `_assert_object_safe` call

**Non-Goals:**

- Changing Git2Provider or VcsProvider trait
- Adding new object-safety checks

## Decisions

### Decision 1: Remove the incorrect size_of_val assertion

**Rationale:** The assertion tests the wrong thing. `_assert_object_safe(&provider)` already proves object safety (code wouldn't compile otherwise). Keeping a redundant, broken assertion adds no value.

**Alternative:** Assert `size_of::<&dyn VcsProvider>() == 2 * size_of::<usize>()` instead—that would be correct but tests a language fact, not our provider. Rejected as unnecessary.

## Risks / Trade-offs

- **None.** Single-line removal, no behavior change to production code.
