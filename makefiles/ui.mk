# makefiles/ui.mk — standardized Make output + requirement helpers.
ESC := $(shell printf '\033')
RESET := $(ESC)[0m
BOLD := $(ESC)[1m
RED := $(ESC)[0;31m

ECHO_ERROR   = printf "$(RED)%s$(RESET)\n" "$(1)"

require_bin = command -v $(1) >/dev/null 2>&1 || { $(ECHO_ERROR) "$(1) is required but not installed"; exit 2; }
require_var = [ -n "$($(1))" ] || { $(ECHO_ERROR) "$(1) is required. Example: make $(MAKECMDGOALS) $(1)=<value>"; exit 2; }
