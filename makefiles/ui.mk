# makefiles/ui.mk — standardized Make output + requirement helpers.
ESC := $(shell printf '\033')
RESET := $(ESC)[0m
BOLD := $(ESC)[1m
RED := $(ESC)[0;31m
GREEN := $(ESC)[0;32m
YELLOW := $(ESC)[0;33m
CYAN := $(ESC)[0;36m

ECHO_INFO    = printf "$(CYAN)%s$(RESET)\n" "$(1)"
ECHO_ERROR   = printf "$(RED)%s$(RESET)\n" "$(1)"
ECHO_SUCCESS = printf "$(GREEN)✓ %s$(RESET)\n" "$(1)"
section      = printf "\n$(BOLD)%s$(RESET)\n" "$(1)"
bullet_ok    = printf "  $(GREEN)✓$(RESET) %s\n" "$(1)"
bullet_fail  = printf "  $(RED)✗$(RESET) %s\n" "$(1)"

require_bin = command -v $(1) >/dev/null 2>&1 || { $(ECHO_ERROR) "$(1) is required but not installed"; exit 2; }
require_var = [ -n "$($(1))" ] || { $(ECHO_ERROR) "$(1) is required. Example: make $(MAKECMDGOALS) $(1)=<value>"; exit 2; }
