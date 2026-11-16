# MCB private handlers for the FLEXT-generated Make surface.
# Public verbs and environment ownership remain in the generated Makefile.
#
# SCOPE RULE: a handler here exists only for what the generated Makefile does
# NOT already own. The new generator policy allows only pre-/post-<verb>
# lifecycle hooks and whole-verb `_custom-<verb>` overrides here — selector
# handlers (`_custom_<verb>_<what>`) are prohibited because canonical verbs
# are already the complete operation. The project-specific surfaces the old
# selectors used to carry now use the declared whole-verb handlers below.

pre-gen:
	@$(PROJECT_FLEXT_INFRA) codegen scaffold --repository-root "$(PROJECT_ROOT)"
	@$(PROJECT_FLEXT_INFRA) codegen conform --root "$(PROJECT_ROOT)" --what lazy-init --scope self --mode apply

post-setup:
	@# Hook installation is owned by beads (`bd hooks install`); Gas City
	@# (gc) owns the workflow, so no recipe here may write .git/hooks.
	@# Why (mcb-o96i.19): CI runners need sccache installed before any cargo
	@# invocation because .cargo/config.toml sets rustc-wrapper = "sccache".
	@# The setup-ci.sh script installs sccache when not present. On local
	@# machines where sccache is already installed this is a no-op.
	@if [ "$$CI" = "Y" ] && [ -f .github/setup-ci.sh ]; then \
		bash .github/setup-ci.sh; \
	fi

pre-check:
	@# Rust gate (mcb-hau9): the generated check surface owns the Python
	@# gates only, so workspace fmt/clippy health enters here — every
	@# `make check` (local, CI=Y and the CI=N complement) fails fast on
	@# formatting drift or clippy warnings. The gate commands have a single
	@# source: scripts/rust/lint.sh, also consumed by the `rust` handler.
	@bash scripts/rust/lint.sh

# The generated docs lifecycle invokes this repository's data-derived metrics
# renderer before validating every documentation artifact.
pre-docs:
	@$(UV_RUN) mcb-scripts metrics

# Why: the generated `build` builtin owns the Python wheel (uv build); the
# Rust workspace binary is this project's artifact, so it builds here as a
# pre-build hook.
pre-build:
	@if [ "$(RELEASE)" = "1" ]; then \
		cargo build --release; \
	else \
		cargo build; \
	fi

_custom-rust:
	@bash scripts/rust/all.sh

_custom-validate:
	@bash scripts/validate/all.sh

_custom-guard:
	@bash scripts/guard/all.sh
