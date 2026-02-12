# git2

Last updated: 2026-02-12

## Usage in MCB

- Repository, branch, commit, and diff operations for VCS context.
- Internal examples: `crates/mcb-providers/src/git/git2_provider.rs`, `crates/mcb-providers/src/git/submodule.rs`.

## Key Capabilities

- Open and inspect repositories (`Repository::open`).
- History traversal with revwalk.
- Local/remote branch and metadata access.

## Best Practices

- Map `git2` errors into domain errors early.
- Handle `NotFound` separately from operational failures.
- Bound history depth for predictable performance.

## Common Pitfalls

- Expensive operations on large worktrees without filtering.
- Assuming a fixed default branch (`main`/`master`) instead of reading `HEAD`.

## Official References

- https://docs.rs/git2
- https://github.com/rust-lang/git2-rs

## GitHub References

- https://github.com/rust-lang/git2-rs/blob/master/src/lib.rs
- https://github.com/gitui-org/gitui/blob/master/asyncgit/src/sync/mod.rs
