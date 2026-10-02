<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# ADR 059: Mimosa Static Advisory Triage and Disposition Policy

<!-- TOC START -->

- [Status](#status)
- [Context](#context)
- [Decision](#decision)
  - [D1: Environment overrides are an operator trust boundary](#d1-environment-overrides-are-an-operator-trust-boundary)
  - [D2: Sink-level containment beats collector-internal guarantees](#d2-sink-level-containment-beats-collector-internal-guarantees)
  - [D3: Vendored fixtures are outside the production dependency graph](#d3-vendored-fixtures-are-outside-the-production-dependency-graph)
  - [D4: Client-side sentinels are not credentials](#d4-client-side-sentinels-are-not-credentials)
  - [D5: Dispositions live in beads, not in the scanner](#d5-dispositions-live-in-beads-not-in-the-scanner)
- [Consequences](#consequences)
- [Alternatives Considered](#alternatives-considered)
- [Implementation Notes](#implementation-notes)
- [References](#references)

<!-- TOC END -->

## Status

Accepted (2026-09-27)

## Context

Sealed Mimosa deep scans (static, no runtime execution) emit cross-file taint advisories
with an explicit proof gap: "static advisory needs human confirmation of the real data
flow and exploitability". Three advisories from
scan-2026-09-24T23-43-42.796Z-63be856b741d were triaged between 2026-09-24 and
2026-09-27, each resolving differently:

1. **HIGH — hardcoded credential, `graphql.rs`**: the GraphQL playground sends a
   placeholder `X-API-Key` header value that the client replaces from `localStorage`
   before use. Disposed via PR #259 (single-sourced sentinel constant, documented as a
   non-credential).
2. **HIGH — path traversal, `pattern_registry/registry.rs`**: the `MCB_RULES_DIR`
   environment override flows into `PatternRegistry::load_from_rules`, which reads every
   collected rule file. Disposed via PR #260: canonical path containment enforced at the
   sink, with a loud warning on every skipped escape, plus a unit test whose layered
   proof (walker forced to follow symlinks, hardening stashed) shows the test fails
   without the sink enforcement and passes with it.
3. **MEDIUM — taint into `Command::new`, `dev/new.rs`**: the sink is a vendored
   third-party fixture under `tests/fixtures/` executing a literal
   `Command::new("git").arg("init")`; the anchor is an MCP handler with no command
   execution. Closed as a false positive with structural evidence.

Sealed re-scans keep reporting the same anchors because the scanner does not track
dispositions (confirmed by the third seal, scan-2026-09-27T18-21-51.664Z-5e255624e733).

## Decision

### D1: Environment overrides are an operator trust boundary

An environment variable documented as an operator deployment knob (such as
`MCB_RULES_DIR`) defines a boundary; it does not escape one. Whoever can set the process
environment already controls the process at equal or higher privilege in every
deployment surface (local dev, CI, Docker, systemd). Such findings are triaged by
documenting the trust model, not by adding privilege boundaries that do not exist.

### D2: Sink-level containment beats collector-internal guarantees

When a taint chain ends at a file-reading sink, the sink enforces its own containment
invariant (canonicalize and compare) instead of relying on a distant collector's
incidental behavior (such as symlink refusal inside a filesystem walker). Violations
skip **with a warning** — silent filtering is a defect under repository law. The
enforcing test must have a demonstrated RED state, not only a GREEN one.

### D3: Vendored fixtures are outside the production dependency graph

Code under `tests/fixtures/` (including vendored third-party trees with their own
manifests) is scan input, not production code. A cross-crate advisory whose sink lives
in a fixture is a false positive when (a) the fixture is not a workspace member or
dependency of any production crate, and (b) the flagged sink takes no external input.
Triage must verify both conditions structurally and record the evidence in the bead.

### D4: Client-side sentinels are not credentials

A placeholder value that the shipped client deterministically replaces before use (and
that grants nothing by itself) is not a credential. It must still be single-sourced as a
named, documented constant so future readers and scanners see intent.

### D5: Dispositions live in beads, not in the scanner

Sealed scans will re-report disposed anchors indefinitely. Each disposition is recorded
on the tracking bead with: the scan id and seal, the structural evidence, the decision
rule (D1–D4 above), and the landing reference (PR or merge commit). A re-reported anchor
with a bead disposition is closed by reference; only new anchors open new beads.

## Consequences

Positive:

- Repeat advisories stop consuming triage effort: the scanner's blind spot is answered
  by a written, auditable disposition trail.
- Security-relevant boundaries (containment at sinks, fixture isolation, sentinel
  naming) are enforced and tested where the advisory pointed, not only argued away.
- New advisories inherit a decision framework instead of ad-hoc reasoning.

Negative:

- Disposed anchors keep appearing in sealed reports and must be recognized by reading
  bead dispositions (a human or agent step per scan).
- Sink-level containment adds code that current collectors already make unreachable
  (accepted defense-in-depth; D2 requires its RED proof to keep it honest).

## Alternatives Considered

- **Suppress scanner paths for disposed files**: rejected — hiding inputs from the
  scanner can hide new findings in the same paths.
- **Patch the vendored fixture**: rejected — mutating vendored third-party fixture code
  breaks its provenance and buys nothing; it executes only as validator test input.
- **Treat environment overrides as vulnerabilities**: rejected — D1; would require
  rejecting every documented deployment knob.

## Implementation Notes

- PR #259 (`c68088ce5`): D4, playground sentinel single-sourcing.
- PR #260 (`ab55a7bc2`): D1+D2, sink containment in
  `crates/mcb-validate/src/pattern_registry/registry.rs`, test
  `crates/mcb-validate/tests/unit/pattern_registry_tests.rs`.
- Bead mcb-ogom: D3 evidence trail (closed false positive).
- Beads mcb-7ttu/mcb-wscq/mcb-ogom carry the scan ids, seals, and per-finding evidence.

## References

- Sealed scan 2026-09-24: `scan-2026-09-24T23-43-42.796Z-63be856b741d` (seal
  `sha256:9f0fa906…`).
- Sealed scan 2026-09-27: `scan-2026-09-27T18-21-51.664Z-5e255624e733` (seal
  `sha256:e2e56f28…`).
- ADR 057 (multi-agent coordination SSOT) — bead tracker as the canonical disposition
  ledger.
