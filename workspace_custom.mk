# Workspace-specific custom targets (never overwritten by sync).
#
# done-check — AGENTS.md §0 R12 / §13 Production-Readiness & Real-User QA.
# Invoked by the Stop hook (~/.agents/hooks/quality-gate.sh, 90s timeout). It must
# be FAST and SCOPED TO THIS SESSION'S COMMITTED WORK — never a whole-workspace or
# fleet gate (§13.4: scope of claim = scope of evidence). It verifies only the
# Python files this branch committed ahead of its upstream (origin/<branch>), so
# other lanes' uncommitted/untracked changes never pollute or brick it. Green when
# nothing is committed-ahead.

.PHONY: done-check
done-check: ## Real-user/green-green check, scoped to committed changes vs upstream
	@base=$$(git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo origin/main); \
	files=$$(git diff --name-only --diff-filter=d "$$base"...HEAD -- '*.py' 2>/dev/null || true); \
	if [ -z "$$files" ]; then \
		echo "done-check: no committed .py changes vs $$base — green/green"; \
		exit 0; \
	fi; \
	n=$$(printf '%s\n' "$$files" | grep -c .); \
	echo "done-check: ruff on $$n committed-vs-$$base .py file(s)"; \
	printf '%s\n' "$$files" | xargs -r ruff check --quiet

.PHONY: gen-agent-pointers
gen-agent-pointers: ## Synchronize generated agent pointer files from AGENTS.md
	@PYTHONPATH=scripts $(MCB_RUN) python scripts/lib/agent_pointers.py $(if $(filter 1,$(CHECK)),--check,)
