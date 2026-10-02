# Documentation automation

`make docs` is the public generation and validation command. The `pre-docs` hook in
`custom.mk` invokes `mcb-scripts metrics`, which reads `config/docs-metrics.toml` and
`docs/templates/metrics.md.j2` to render `docs/generated/METRICS.md`. Repeating
`make docs` with unchanged inputs must leave the generated file unchanged.

`mcb-scripts adr --title "Decision Title"` renders `docs/templates/adr-template.md` into
a new numbered ADR. Its title and status are validated, and publication never replaces
an existing record. Use `--dry-run` to see the destination.

The remaining shell scripts under this directory support project-specific validation and
diagram tasks. They do not own metrics or ADR generation.
