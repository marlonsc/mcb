#!/usr/bin/env bash
set -euo pipefail

if ! git rev-parse --git-dir >/dev/null 2>&1; then
	echo "Not a git repository."
	exit 1
fi

if ! git diff --cached --quiet; then
	staged_files=$(git diff --cached --name-only)
else
	echo "No staged changes. Stage files before running."
	exit 1
fi

branch_name=$(git branch --show-current 2>/dev/null || true)
issue_id=""
if [[ -n "${branch_name}" ]]; then
	issue_id=$(echo "${branch_name}" | grep -oE '[a-z]+-[a-z0-9]+' | head -1 || true)
fi

is_only_docs=true
is_only_tests=true
is_only_ops=true
has_code_changes=false

declare -A scope_counts=()

while IFS= read -r file; do
	case "${file}" in
	docs/* | *.md)
		is_only_tests=false
		is_only_ops=false
		;;
	*/tests/* | */test/* | *_test.*)
		is_only_docs=false
		is_only_ops=false
		;;
	scripts/* | make/* | Makefile | .pre-commit-config.yaml | .github/*)
		is_only_docs=false
		is_only_tests=false
		;;
	crates/*/src/*)
		is_only_docs=false
		is_only_tests=false
		is_only_ops=false
		has_code_changes=true
		;;
	*)
		is_only_docs=false
		is_only_tests=false
		is_only_ops=false
		;;
	esac

	if [[ "${file}" == crates/*/* ]]; then
		crate_name=$(echo "${file}" | cut -d'/' -f2)
		current_count=${scope_counts["${crate_name}"]:-0}
		scope_counts["${crate_name}"]=$((current_count + 1))
	else
		top_level=$(echo "${file}" | cut -d'/' -f1)
		current_count=${scope_counts["${top_level}"]:-0}
		scope_counts["${top_level}"]=$((current_count + 1))
	fi
done <<<"${staged_files}"

suggested_type="chore"
if [[ "${is_only_docs}" == true ]]; then
	suggested_type="docs"
elif [[ "${is_only_tests}" == true ]]; then
	suggested_type="test"
elif [[ "${is_only_ops}" == true ]]; then
	suggested_type="chore"
elif [[ "${has_code_changes}" == true ]]; then
	suggested_type="feat"
fi

suggested_scope=""
if [[ ${#scope_counts[@]} -gt 0 ]]; then
	suggested_scope=$(for scope in "${!scope_counts[@]}"; do
		echo "${scope_counts[${scope}]} ${scope}"
	done | sort -rn | head -1 | awk '{print $2}')
fi

stat_summary=$(git diff --cached --stat | tail -1 | sed 's/^ //')

echo "Suggested type: ${suggested_type}"
if [[ -n "${suggested_scope}" ]]; then
	echo "Suggested scope: ${suggested_scope}"
fi
echo "Summary: ${stat_summary}"

if [[ -n "${suggested_scope}" ]]; then
	echo "Subject: ${suggested_type}(${suggested_scope}): <short description>"
else
	echo "Subject: ${suggested_type}: <short description>"
fi

if [[ -n "${issue_id}" ]]; then
	echo "Footer: Fixes #${issue_id}"
fi
