# Proposal: Fix git2 object-safe test

## Why

`test_git2_provider_is_object_safe` in `mcb-providers` fails with `left: 0, right: 16`. The test asserts `std::mem::size_of_val(erased) == 2 * std::mem::size_of::<usize>()`, but `size_of_val` returns the size of the referent (the underlying object), not the size of the trait object reference. The assertion is incorrect and redundant—object safety is already proven by `_assert_object_safe(&provider)` compiling and running.

## What Changes

- Remove or correct the incorrect `size_of_val` assertion in `test_git2_provider_is_object_safe`
- Test continues to verify object safety via the `_assert_object_safe(&provider)` call

## Capabilities

### New Capabilities

- `git2-provider-object-safety`: Git2Provider VcsProvider implementation is verified object-safe via a test that compiles and runs successfully

### Modified Capabilities

<!-- None -->

## Impact

- `crates/mcb-providers/tests/unit/git2_provider_tests.rs`: One test function modified
- No API or dependency changes
