# =============================================================================
# mcb-scripts — private custom handlers only
# =============================================================================
# Public Make surface is the flext-infra generated Makefile. Domain actions
# attach as `_custom_<verb>_<what>` and preserve the former boot/build/test/
# check/ship/clean dispatch via makefiles/domain.mk.
#
# Migration map (former → flext):
#   boot   → setup WHAT=...
#   build  → build WHAT=...
#   test   → test WHAT=...  (SCOPE= passed through)
#   check  → check WHAT=...
#   ship   → work/status/release WHAT=...
#   clean  → clean WHAT=...
# =============================================================================

include makefiles/domain.mk

# --- internal dispatch helpers ------------------------------------------------
define _mcb_call
$(MAKE) --no-print-directory _mcb_internal_$(1) WHAT=$(2) $(3)
endef

# --- run (former boot + dev server / docker) ----------------------------------
.PHONY: _custom_run_mcb-hooks _custom_run_mcb-hook-pre-commit _custom_run_mcb-hook-pre-push
.PHONY: _custom_run_mcb-tools _custom_run_mcb-adr _custom_run_mcb-venv _custom_run_mcb-all
.PHONY: _custom_run_dev _custom_run_dev-run _custom_run_docker-up
.PHONY: _custom_run_docker-down _custom_run_docker-logs _custom_run_docker-test

_custom_run_mcb-hooks: ## Install git hooks (former boot WHAT=hooks)
	@$(call _mcb_call,boot,hooks)

_custom_run_mcb-hook-pre-commit: ## Run pre-commit hook gate
	@$(call _mcb_call,boot,hook,ACT=pre-commit)

_custom_run_mcb-hook-pre-push: ## Run pre-push hook gate
	@$(call _mcb_call,boot,hook,ACT=pre-push)

_custom_run_mcb-tools: ## Install Rust/Python tooling (former boot WHAT=tools)
	@$(call _mcb_call,boot,tools)

_custom_run_mcb-adr: ## Install ADR tooling (former boot WHAT=adr)
	@$(call _mcb_call,boot,adr)

_custom_run_mcb-venv: ## Sync Python extras beyond flext setup (former boot WHAT=venv)
	@$(call _mcb_call,boot,venv)

_custom_run_mcb-all: ## Full MCB bootstrap after flext setup (former boot WHAT=all)
	@$(call _mcb_call,boot,all)

post-setup: ## After flext environment: install MCB git hooks
	@$(MAKE) --no-print-directory run WHAT=mcb-hooks

_custom_run_dev: ; @$(call _mcb_call,check,dev,ACT=$(or $(ACT),run) APPLY=$(APPLY))
_custom_run_dev-run: ; @$(call _mcb_call,check,dev,ACT=run APPLY=$(APPLY))
_custom_run_docker-up: ; @$(call _mcb_call,check,dev,ACT=docker-up APPLY=$(APPLY))
_custom_run_docker-down: ; @$(call _mcb_call,check,dev,ACT=docker-down APPLY=$(APPLY))
_custom_run_docker-logs: ; @$(call _mcb_call,check,dev,ACT=docker-logs APPLY=$(APPLY))
_custom_run_docker-test: ; @$(call _mcb_call,check,dev,ACT=docker-test APPLY=$(APPLY))

# --- build (Rust/cargo/codegen/docs) ------------------------------------------
.PHONY: _custom_build_build _custom_build_debug _custom_build_release
.PHONY: _custom_build_prebuild _custom_build_codegen _custom_build_codegen-cli
.PHONY: _custom_build_codegen-db _custom_build_codegen-entities
.PHONY: _custom_build_codegen-conversions _custom_build_codegen-clean
.PHONY: _custom_build_codegen-all _custom_build_docs _custom_build_docs-build
.PHONY: _custom_build_docs-serve _custom_build_docs-lint _custom_build_docs-validate
.PHONY: _custom_build_docs-sync _custom_build_docs-rust _custom_build_docs-check
.PHONY: _custom_build_docs-setup _custom_build_docs-adr _custom_build_docs-adr-new
.PHONY: _custom_build_docs-diagrams _custom_build_artifacts

_custom_build_build: ## Cargo build (RELEASE=0|1)
	@$(call _mcb_call,build,build)

_custom_build_debug: ## Cargo debug build
	@$(call _mcb_call,build,debug)

_custom_build_release: ## Cargo release build
	@$(call _mcb_call,build,release)

_custom_build_prebuild: ## Pre-build all test targets
	@$(call _mcb_call,build,prebuild)

_custom_build_artifacts: ## Default build (honors RELEASE=)
	@$(call _mcb_call,build,build)

_custom_build_codegen: ## SeaORM codegen (ACT= selects phase)
	@$(call _mcb_call,build,codegen,ACT=$(ACT) APPLY=$(APPLY))

_custom_build_codegen-cli: ; @$(call _mcb_call,build,codegen,ACT=cli APPLY=$(APPLY))
_custom_build_codegen-db: ; @$(call _mcb_call,build,codegen,ACT=db APPLY=$(APPLY))
_custom_build_codegen-entities: ; @$(call _mcb_call,build,codegen,ACT=entities APPLY=$(APPLY))
_custom_build_codegen-conversions: ; @$(call _mcb_call,build,codegen,ACT=conversions APPLY=$(APPLY))
_custom_build_codegen-clean: ; @$(call _mcb_call,build,codegen,ACT=clean APPLY=$(APPLY))
_custom_build_codegen-all: ; @$(call _mcb_call,build,codegen,ACT=all APPLY=$(APPLY))

_custom_build_docs: ## Docs pipeline (ACT= selects phase)
	@$(call _mcb_call,build,docs,ACT=$(ACT) QUICK=$(QUICK) FIX=$(FIX) APPLY=$(APPLY))

_custom_build_docs-build: ; @$(call _mcb_call,build,docs,ACT=build APPLY=$(APPLY))
_custom_build_docs-serve: ; @$(call _mcb_call,build,docs,ACT=serve APPLY=$(APPLY))
_custom_build_docs-lint: ; @$(call _mcb_call,build,docs,ACT=lint FIX=$(FIX) APPLY=$(APPLY))
_custom_build_docs-validate: ; @$(call _mcb_call,build,docs,ACT=validate QUICK=$(QUICK))
_custom_build_docs-sync: ; @$(call _mcb_call,build,docs,ACT=sync APPLY=$(APPLY))
_custom_build_docs-rust: ; @$(call _mcb_call,build,docs,ACT=rust)
_custom_build_docs-check: ; @$(call _mcb_call,build,docs,ACT=check)
_custom_build_docs-setup: ; @$(call _mcb_call,build,docs,ACT=setup APPLY=$(APPLY))
_custom_build_docs-adr: ; @$(call _mcb_call,build,docs,ACT=adr)
_custom_build_docs-adr-new: ; @$(call _mcb_call,build,docs,ACT=adr-new APPLY=$(APPLY))
_custom_build_docs-diagrams: ; @$(call _mcb_call,build,docs,ACT=diagrams APPLY=$(APPLY))

# --- test (Rust/cargo-nextest; SCOPE= selects suite) -------------------------
.PHONY: _custom_test_unit _custom_test_doc _custom_test_golden _custom_test_startup
.PHONY: _custom_test_warmup _custom_test_integration _custom_test_external
.PHONY: _custom_test_changed _custom_test_e2e _custom_test_all

_custom_test_unit: ; @$(call _mcb_call,test,,SCOPE=unit THREADS=$(THREADS))
_custom_test_doc: ; @$(call _mcb_call,test,,SCOPE=doc THREADS=$(THREADS))
_custom_test_golden: ; @$(call _mcb_call,test,,SCOPE=golden THREADS=$(THREADS))
_custom_test_startup: ; @$(call _mcb_call,test,,SCOPE=startup THREADS=$(THREADS))
_custom_test_warmup: ; @$(call _mcb_call,test,,SCOPE=warmup THREADS=$(THREADS))
_custom_test_integration: ; @$(call _mcb_call,test,,SCOPE=integration THREADS=$(THREADS))
_custom_test_external: ; @$(call _mcb_call,test,,SCOPE=external THREADS=$(THREADS))
_custom_test_changed: ; @$(call _mcb_call,test,,SCOPE=changed THREADS=$(THREADS))
_custom_test_e2e: ; @$(call _mcb_call,test,,SCOPE=e2e THREADS=$(THREADS) APPLY=$(APPLY))
_custom_test_all: ## Full Rust test suite (+ e2e when SCOPE=all)
	@$(call _mcb_call,test,,SCOPE=all THREADS=$(THREADS) APPLY=$(APPLY))

# --- check (gates, fix, dev, python, ci) ------------------------------------
.PHONY: _custom_check_fmt _custom_check_lint _custom_check_validate
.PHONY: _custom_check_audit _custom_check_udeps _custom_check_coverage
.PHONY: _custom_check_qlty _custom_check_coordination _custom_check_guard
.PHONY: _custom_check_gitops _custom_check_surface _custom_check_python
.PHONY: _custom_check_python-lint _custom_check_python-lint-staged
.PHONY: _custom_check_python-test _custom_check_python-test-staged
.PHONY: _custom_check_python-guard _custom_check_python-all
.PHONY: _custom_check_fix _custom_check_fix-fmt _custom_check_fix-lint
.PHONY: _custom_check_fix-docs _custom_check_fix-all
.PHONY: _custom_check_dev _custom_check_dev-run _custom_check_dev-docker-up
.PHONY: _custom_check_dev-docker-down _custom_check_dev-docker-logs
.PHONY: _custom_check_dev-docker-test _custom_check_optimize _custom_check_optimize-cache
.PHONY: _custom_check_ci _custom_check_mcb-ci

_custom_check_fmt: ; @$(call _mcb_call,check,fmt)
_custom_check_lint: ; @$(call _mcb_call,check,lint)
_custom_check_validate: ; @$(call _mcb_call,check,validate,QUICK=$(QUICK))
_custom_check_audit: ; @$(call _mcb_call,check,audit)
_custom_check_udeps: ; @$(call _mcb_call,check,udeps)
_custom_check_coverage: ; @$(call _mcb_call,check,coverage)
_custom_check_qlty: ; @$(call _mcb_call,check,qlty)
_custom_check_coordination: ; @$(call _mcb_call,check,coordination)
_custom_check_guard: ; @$(call _mcb_call,check,guard)
_custom_check_gitops: ; @$(call _mcb_call,check,gitops)
_custom_check_surface: ; @$(call _mcb_call,check,surface)
_custom_check_python: ; @$(call _mcb_call,check,python,ACT=$(ACT) QUICK=$(QUICK))
_custom_check_python-lint: ; @$(call _mcb_call,check,python,ACT=lint)
_custom_check_python-lint-staged: ; @$(call _mcb_call,check,python,ACT=lint-staged)
_custom_check_python-test: ; @$(call _mcb_call,check,python,ACT=test)
_custom_check_python-test-staged: ; @$(call _mcb_call,check,python,ACT=test-staged)
_custom_check_python-guard: ; @$(call _mcb_call,check,python,ACT=guard)
_custom_check_python-all: ; @$(call _mcb_call,check,python,ACT=all)
_custom_check_fix: ; @$(call _mcb_call,check,fix,ACT=$(ACT) APPLY=$(APPLY))
_custom_check_fix-fmt: ; @$(call _mcb_call,check,fix,ACT=fmt APPLY=$(APPLY))
_custom_check_fix-lint: ; @$(call _mcb_call,check,fix,ACT=lint APPLY=$(APPLY))
_custom_check_fix-docs: ; @$(call _mcb_call,check,fix,ACT=docs APPLY=$(APPLY))
_custom_check_fix-all: ; @$(call _mcb_call,check,fix,ACT=all APPLY=$(APPLY))
_custom_check_dev: ; @$(call _mcb_call,check,dev,ACT=$(ACT) APPLY=$(APPLY))
_custom_check_dev-run: ; @$(call _mcb_call,check,dev,ACT=run APPLY=$(APPLY))
_custom_check_dev-docker-up: ; @$(call _mcb_call,check,dev,ACT=docker-up APPLY=$(APPLY))
_custom_check_dev-docker-down: ; @$(call _mcb_call,check,dev,ACT=docker-down APPLY=$(APPLY))
_custom_check_dev-docker-logs: ; @$(call _mcb_call,check,dev,ACT=docker-logs APPLY=$(APPLY))
_custom_check_dev-docker-test: ; @$(call _mcb_call,check,dev,ACT=docker-test APPLY=$(APPLY))
_custom_check_optimize: ; @$(call _mcb_call,check,optimize,ACT=$(ACT) APPLY=$(APPLY))
_custom_check_optimize-cache: ; @$(call _mcb_call,check,optimize,ACT=cache APPLY=$(APPLY))
_custom_check_ci: ## Full MCB CI gate (former check WHAT=ci|all)
	@$(call _mcb_call,check,ci,QUICK=$(QUICK))
_custom_check_mcb-ci: ; @$(call _mcb_call,check,ci,QUICK=$(QUICK))

# --- run (dev server — also listed under check WHAT=dev) ----------------------
# --- status (read-only git — former ship WHAT=status|diff|log|show) -----------
.PHONY: _custom_status_git _custom_status_diff _custom_status_log _custom_status_show

_custom_status_git: ; @$(call _mcb_call,ship,status)
_custom_status_diff: ; @$(call _mcb_call,ship,diff)
_custom_status_log: ; @$(call _mcb_call,ship,log,LOG_N=$(LOG_N))
_custom_status_show: ; @$(call _mcb_call,ship,show,REF=$(REF))

# --- work (mutating git/PR/sub — former ship) --------------------------------
.PHONY: _custom_work_add _custom_work_commit _custom_work_push _custom_work_pull
.PHONY: _custom_work_branch _custom_work_checkout _custom_work_tag _custom_work_tags
.PHONY: _custom_work_stash _custom_work_stash-pop _custom_work_stash-list
.PHONY: _custom_work_merge _custom_work_rebase _custom_work_unstage
.PHONY: _custom_work_push-tags _custom_work_pr _custom_work_pr-checks
.PHONY: _custom_work_pr-view _custom_work_pr-merge _custom_work_pr-rerun
.PHONY: _custom_work_sub _custom_work_sub-status

_custom_work_add: ; @$(call _mcb_call,ship,add,FILES="$(FILES)" APPLY=$(APPLY))
_custom_work_commit: ; @$(call _mcb_call,ship,commit,MSG="$(MSG)" FILES="$(FILES)" APPLY=$(APPLY))
_custom_work_push: ; @$(call _mcb_call,ship,push,BRANCH=$(BRANCH) APPLY=$(APPLY))
_custom_work_pull: ; @$(call _mcb_call,ship,pull,BRANCH=$(BRANCH) APPLY=$(APPLY))
_custom_work_branch: ; @$(call _mcb_call,ship,branch,REF=$(REF) BASE=$(BASE) APPLY=$(APPLY))
_custom_work_checkout: ; @$(call _mcb_call,ship,checkout,REF=$(REF) APPLY=$(APPLY))
_custom_work_tag: ; @$(call _mcb_call,ship,tag,TAG=$(TAG) MSG="$(MSG)" APPLY=$(APPLY))
_custom_work_tags: ; @$(call _mcb_call,ship,tags)
_custom_work_stash: ; @$(call _mcb_call,ship,stash,MSG="$(MSG)" APPLY=$(APPLY))
_custom_work_stash-pop: ; @$(call _mcb_call,ship,stash-pop,APPLY=$(APPLY))
_custom_work_stash-list: ; @$(call _mcb_call,ship,stash-list)
_custom_work_merge: ; @$(call _mcb_call,ship,merge,REF=$(REF) APPLY=$(APPLY))
_custom_work_rebase: ; @$(call _mcb_call,ship,rebase,BASE=$(BASE) APPLY=$(APPLY))
_custom_work_unstage: ; @$(call _mcb_call,ship,unstage,FILES="$(FILES)" APPLY=$(APPLY))
_custom_work_push-tags: ; @$(call _mcb_call,ship,push-tags,TAG=$(TAG) APPLY=$(APPLY))
_custom_work_pr: ; @$(call _mcb_call,ship,pr,ACT=$(ACT) PR=$(PR) RUN=$(RUN) APPLY=$(APPLY))
_custom_work_pr-checks: ; @$(call _mcb_call,ship,pr,ACT=checks PR=$(PR))
_custom_work_pr-view: ; @$(call _mcb_call,ship,pr,ACT=view PR=$(PR))
_custom_work_pr-merge: ; @$(call _mcb_call,ship,pr,ACT=merge PR=$(PR) APPLY=$(APPLY))
_custom_work_pr-rerun: ; @$(call _mcb_call,ship,pr,ACT=rerun RUN=$(RUN) APPLY=$(APPLY))
_custom_work_sub: ; @$(call _mcb_call,ship,sub,ACT=$(ACT) SUB=$(SUB) MSG="$(MSG)" APPLY=$(APPLY))
_custom_work_sub-status: ; @$(call _mcb_call,ship,sub,ACT=status)

# --- release (former ship WHAT=release) ---------------------------------------
.PHONY: _custom_release_package _custom_release_version _custom_release_install
.PHONY: _custom_release_install-validate _custom_release_mcb

_custom_release_package: ; @$(call _mcb_call,ship,release,ACT=package APPLY=$(APPLY))
_custom_release_version: ; @$(call _mcb_call,ship,release,ACT=version BUMP=$(BUMP) APPLY=$(APPLY))
_custom_release_install: ; @$(call _mcb_call,ship,release,ACT=install APPLY=$(APPLY))
_custom_release_install-validate: ; @$(call _mcb_call,ship,release,ACT=install-validate)
_custom_release_mcb: ; @$(call _mcb_call,ship,release,ACT=$(or $(ACT),package) APPLY=$(APPLY))

# --- clean --------------------------------------------------------------------
.PHONY: _custom_clean_build _custom_clean_codegen _custom_clean_all _custom_clean_mcb

_custom_clean_build: ; @$(call _mcb_call,clean,build,APPLY=$(APPLY))
_custom_clean_codegen: ; @$(call _mcb_call,clean,codegen,APPLY=$(APPLY))
_custom_clean_all: ; @$(call _mcb_call,clean,all,APPLY=$(APPLY))
_custom_clean_mcb: ; @$(call _mcb_call,clean,all,APPLY=$(APPLY))

# --- help / workspace hooks ---------------------------------------------------
.PHONY: _custom_help_mcb _custom_help_all

_custom_help_mcb:
	@printf '\nMCB domain WHAT selectors (via flext public verbs):\n'
	@printf '  run:      mcb-hooks mcb-tools mcb-adr mcb-venv mcb-all mcb-hook-pre-commit\n'
	@printf '  setup:    flext environment (+ post-setup installs MCB hooks)\n'
	@printf '  build:    build debug release prebuild codegen[-*] docs[-*]\n'
	@printf '  test:     unit doc golden startup warmup integration external changed e2e all\n'
	@printf '  check:    fmt lint validate audit ci python[-*] fix[-*] dev[-*] optimize-cache\n'
	@printf '  work:     add commit push pull branch tag stash merge pr[-*] sub-status\n'
	@printf '  release:  package version install install-validate\n'
	@printf '  clean:    build codegen all\n'

_custom_help_all:
	$(Q)$(MAKE) --no-print-directory _builtin_help_usage
	@$(MAKE) --no-print-directory _custom_help_mcb

.PHONY: done-check gen-agent-pointers
done-check: ## Real-user check scoped to committed .py vs upstream
	@base=$$(git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo origin/main); \
	files=$$(git diff --name-only --diff-filter=d "$$base"...HEAD -- '*.py' 2>/dev/null || true); \
	if [ -z "$$files" ]; then \
		echo "done-check: no committed .py changes vs $$base — green/green"; \
		exit 0; \
	fi; \
	n=$$(printf '%s\n' "$$files" | grep -c .); \
	echo "done-check: ruff on $$n committed-vs-$$base .py file(s)"; \
	printf '%s\n' "$$files" | xargs -r ruff check --quiet

gen-agent-pointers: ## Synchronize generated agent pointer files from AGENTS.md
	@UV_CACHE_DIR=.cache/uv $(MCB_RUN) python scripts/lib/agent_pointers.py $(if $(filter 1,$(CHECK)),--check,)
