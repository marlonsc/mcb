# =============================================================================
# ALIASES - Single-letter shortcuts for power users
# =============================================================================
# Usage: make <letter>
# =============================================================================

# Core workflow
b: build      ## Build (debug)
t: test       ## Test all
c: check      ## Check compilation
f: fix        ## Auto-fix issues
l: lint       ## Lint all code
q: quality    ## Full quality check

# Development
r: run        ## Run server
d: dev        ## Dev server (watch)
s: status     ## Git status

# Documentation
D: docs       ## Build docs
S: docs-serve ## Serve docs

# Git operations
y: sync       ## Sync to remote
p: push       ## Push only

# Maintenance
u: update     ## Update deps
a: audit      ## Security audit
