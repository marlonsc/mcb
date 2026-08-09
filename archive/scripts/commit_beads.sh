#!/usr/bin/env bash
set -euo pipefail

commit_message=${1:-}
commit_hash=${2:-}
branch_name=${3:-}

if [[ -z "${commit_hash}" ]]; then
	commit_hash=$(git rev-parse HEAD 2>/dev/null || true)
fi

if [[ -z "${commit_message}" ]]; then
	commit_message=$(git log -1 --pretty=%B 2>/dev/null || true)
fi

if [[ -z "${branch_name}" ]]; then
	branch_name=$(git branch --show-current 2>/dev/null || true)
fi

if [[ -z "${commit_message}" || -z "${commit_hash}" ]]; then
	echo "Usage: $0 \"<commit_message>\" <commit_hash> [branch_name]"
	exit 1
fi

issue_id=""
if [[ -n "${branch_name}" ]]; then
	issue_id=$(echo "${branch_name}" | grep -oE '[a-z]+-[a-z0-9]+' | head -1 || true)
fi

footer_issue=""
footer_issue=$(echo "${commit_message}" | grep -oiE '(fixes|closes) #[a-z0-9-]+' | head -1 | awk '{print $2}' | tr -d '#') || true

if [[ -n "${footer_issue}" ]]; then
	bd close "${footer_issue}" --reason "Closed by commit ${commit_hash}" --json >/dev/null
	exit 0
fi

if [[ -n "${issue_id}" ]]; then
	subject_line=$(echo "${commit_message}" | head -1)
	bd update "${issue_id}" --notes "Commit ${commit_hash}: ${subject_line}" --json >/dev/null
fi
