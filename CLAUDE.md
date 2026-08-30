<!-- AIHUB-INVIOLABLE-LAW-PRELUDE v1 -->
# AI Hub Inviolable Law — Strict Prelude

1. Truth: never claim done/green/resolved without command, exit code, decisive output.
2. Root cause: no bypass, fallback, shim, suppression, stub, hardcode, or old+new coexistence.
3. Beads first: claim/update bead before file write, shell, or multi-step work; update after every repo-state change.
4. Research first: inspect code, docs, canonical sources before acting; never invent APIs, flags, facts, or behavior.
5. Owner first: use the project's declared facades/primitives; do not reimplement them locally.
6. Gate discipline: if a gate blocks, stop and escalate with the exact command/edit; never route around it.
7. Landing: native gates, commit, fast-forward push, bead evidence.
8. Divergence: FF push rejected → integrate by cooperation: `git merge --no-ff` the integration base into your lane, resolve conflicts, revalidate, land. Never rebase or force-push a shared branch; never discard another actor's work.
9. Escalation: impossible rule → exact error. Rule conflict → present both with numbers. Unclear → one targeted question. Never guess.
10. Precedence: NEWEST > OLDEST. USER REQUEST > BEADS > ADRs > SKILLs > DOCS > default. Adjust lower/older to higher/newer. Doubt → ASK USER FIRST.
<!-- /AIHUB-INVIOLABLE-LAW-PRELUDE -->

## CLAUDE.md

Canonical governance lives in this repo's `AGENTS.md`, whose first bytes are
the strict prelude selected by `config.AiHub.governance.law_surface`, followed
by the project overlay. Generated universal-core bodies are retired. **Do not
duplicate rules here** — keep only project-specific notes below.

<!-- AIHUB-POINTER-AUTHORITY-BEGIN -->
<!-- AIHUB-POINTER-AUTHORITY-END -->

- **Task tracking:** `bd` (beads). Run `bd prime`.
- **Validation:** prefer `make` targets (`make lint` / `make typecheck` /
  `make test`).
- **Tools:** `ast-grep` (`sg`) for structural search; never `rm` / `sed -i`
  (use the Edit tool or `trash-put`).

<!-- project-specific notes below -->
