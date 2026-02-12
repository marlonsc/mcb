# Git Workflow

Last updated: 2026-02-11

Working practices:

- Inspect status before and after changes.
- Keep commits atomic and conventionally named.
- Avoid destructive history operations unless explicitly requested.

Beads alignment:

- Move selected issue to `in_progress` before implementation.
- Close issue when work is verified and complete.
- Run `bd sync` to keep issue state aligned with git history.

Pre-merge checks:

- Run project tests and build/typecheck where applicable.
- Ensure no temporary debug code remains.
