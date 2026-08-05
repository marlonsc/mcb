#!/usr/bin/env bash
# /// cosmos-command
# verb = "check"
# what = "all"
# domain = "quality"
# summary = "Run the workspace's default check gate"
# description = "Delegates to the existing 'make check' target in the main Makefile."
# example = "make check WHAT=all"
# mutates = false
# ///
set -euo pipefail
exec make check
