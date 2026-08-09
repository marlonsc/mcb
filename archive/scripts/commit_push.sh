#!/usr/bin/env bash
set -euo pipefail

if ! git rev-parse --git-dir >/dev/null 2>&1; then
	echo "Not a git repository."
	exit 1
fi

default_remote=$(git remote | head -1)
current_branch=$(git branch --show-current 2>/dev/null || true)

if [[ -z "${default_remote}" || -z "${current_branch}" ]]; then
	echo "Unable to determine remote/branch."
	exit 1
fi

if [[ ! -t 0 ]]; then
	echo "No TTY available. Skipping push confirmation."
	exit 0
fi

read -r -p "Push now? [y/N]: " confirm
confirm=${confirm:-N}
if [[ ! "${confirm}" =~ ^[Yy]$ ]]; then
	echo "Skipping push."
	exit 0
fi

read -r -p "Remote [${default_remote}]: " remote
remote=${remote:-${default_remote}}

read -r -p "Branch [${current_branch}]: " branch
branch=${branch:-${current_branch}}

git push "${remote}" "${branch}"
