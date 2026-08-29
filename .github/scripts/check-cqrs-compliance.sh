#!/usr/bin/env bash
# Owner-Skill: .agents/skills/scripts-validation/SKILL.md
# check-cqrs-compliance.sh — Enforce strict CQRS/FlextModels patterns across the ecosystem.
#
# Prevents:
#   1. Command classes inheriting from BaseModel instead of m.Command
#   2. Event classes inheriting from BaseModel instead of m.DomainEvent/m.Event
#   3. setattr hacks for message_type (handlers must be self-describing)
#   4. Direct FlextDispatcher() instantiation (must use p.Dispatcher via DI)
#   5. Query classes inheriting from BaseModel instead of m.Query
#
# Exclusions:
#   - flext-ldif/       (ValidationLevel is domain-specific, not CQRS)
#   - flext-core/src/   (defines the base classes themselves)
#   - examples/         (pedagogical code)
#   - tests/            (only checked for setattr hacks, not BaseModel usage)
#
# Usage:
#   make check-cqrs-compliance
#   .github/scripts/check-cqrs-compliance.sh [--verbose]

set -euo pipefail

VERBOSE="${1:-}"
WORKSPACE_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VIOLATIONS=0
SOURCE_ROOTS=()
TEST_ROOTS=()
cd "${WORKSPACE_ROOT}"
shopt -s nullglob
for path in ./*/src; do
	if [[ -L "${path}" || ! -d "${path}" ]]; then
		printf 'ERROR: invalid workspace source root: %s\n' "${path}" >&2
		exit 2
	fi
	SOURCE_ROOTS+=("${path}")
done
for path in ./*/tests; do
	if [[ -L "${path}" || ! -d "${path}" ]]; then
		printf 'ERROR: invalid workspace test root: %s\n' "${path}" >&2
		exit 2
	fi
	TEST_ROOTS+=("${path}")
done

_log() {
	if [[ "${VERBOSE}" == "--verbose" ]]; then
		echo "[INFO] $*"
	fi
}

_fail() {
	local check="$1" pattern="$2" file="$3" line="$4"
	echo "VIOLATION [${check}]: ${file}:${line} — ${pattern}"
	VIOLATIONS=$((VIOLATIONS + 1))
}

_capture_rg() {
	local pattern="$1"
	shift
	if [[ "$#" -eq 0 ]]; then return 0; fi
	local output status
	if output=$(rg -n --glob '*.py' --glob '!**/__pycache__/**' "${pattern}" "$@"); then
		printf '%s\n' "${output}"
		return 0
	else
		status=$?
	fi
	if [[ "${status}" -eq 1 ]]; then return 0; fi
	return "${status}"
}

_check() {
	local check="$1" pattern="$2" description="$3"
	shift 3
	local exclude_patterns=("$@")

	_log "Checking: ${description}"

	local matches
	matches=$(_capture_rg "${pattern}" "${SOURCE_ROOTS[@]}")

	if [[ -n "${matches}" ]]; then
		while IFS= read -r match; do
			local excluded=0
			for excl in "${exclude_patterns[@]}"; do
				if [[ "${match}" == "./${excl}/"* ]]; then excluded=1; break; fi
			done
			if [[ "${excluded}" -eq 1 ]]; then continue; fi
			local file line_content
			file=$(echo "${match}" | cut -d: -f1)
			local line_num
			line_num=$(echo "${match}" | cut -d: -f2)
			line_content=$(echo "${match}" | cut -d: -f3-)
			_fail "${check}" "${line_content}" "${file}" "${line_num}"
		done <<<"${matches}"
	else
		_log "  PASS: ${description}"
	fi
}

echo "=== CQRS Compliance Check ==="
echo ""

# 1. No Command(m.BaseModel) in production code
_check "CMD-BASEMODEL" \
	'class.*Command(m.BaseModel)' \
	"Command classes must inherit from m.Command, not BaseModel" \
	"flext-ldif" "flext-core" "examples" "__pycache__"

# 2. No Event(m.BaseModel) in production code
_check "EVT-BASEMODEL" \
	'class.*Event(m.BaseModel)' \
	"Event classes must inherit from m.DomainEvent or m.Event, not BaseModel" \
	"flext-ldif" "flext-core" "examples" "__pycache__"

# 3. No setattr hacks for message_type (prod AND test code)
_log "Checking: setattr hacks for message_type (prod + tests)"
matches=$(_capture_rg 'setattr.*message_type' "${SOURCE_ROOTS[@]}" "${TEST_ROOTS[@]}")
if [[ -n "${matches}" ]]; then
	while IFS= read -r match; do
		if [[ "${match}" == "./flext-ldif/"* || "${match}" == "./examples/"* ]]; then
			continue
		fi
		file=$(echo "${match}" | cut -d: -f1)
		line_num=$(echo "${match}" | cut -d: -f2)
		line_content=$(echo "${match}" | cut -d: -f3-)
		_fail "SETATTR-HACK" "${line_content}" "${file}" "${line_num}"
	done <<<"${matches}"
else
	_log "  PASS: No setattr message_type hacks"
fi

# 4. No direct FlextDispatcher() instantiation (except container registration)
_log "Checking: Direct FlextDispatcher() instantiation"
matches=$(_capture_rg '= FlextDispatcher\(\)' "${SOURCE_ROOTS[@]}")
if [[ -n "${matches}" ]]; then
	while IFS= read -r match; do
		if [[ "${match}" == "./flext-ldif/"* || "${match}" == "./examples/"* || "${match}" == *container* || "${match}" == *register* ]]; then
			continue
		fi
		file=$(echo "${match}" | cut -d: -f1)
		line_num=$(echo "${match}" | cut -d: -f2)
		line_content=$(echo "${match}" | cut -d: -f3-)
		_fail "DIRECT-DISPATCHER" "${line_content}" "${file}" "${line_num}"
	done <<<"${matches}"
else
	_log "  PASS: No direct FlextDispatcher() instantiation"
fi

# 5. No Query(m.BaseModel) in production code
_check "QRY-BASEMODEL" \
	'class.*Query(m.BaseModel)' \
	"Query classes must inherit from m.Query, not BaseModel" \
	"flext-ldif" "flext-core" "examples" "__pycache__"

echo ""
if [[ "${VIOLATIONS}" -eq 0 ]]; then
	echo "✅ CQRS Compliance: ALL CHECKS PASSED (0 violations)"
	exit 0
else
	echo "❌ CQRS Compliance: FAILED (${VIOLATIONS} violation(s))"
	echo ""
	echo "Fix instructions:"
	echo "  CMD-BASEMODEL     → Change class MyCommand(m.BaseModel) to class MyCommand(m.Command)"
	echo "  EVT-BASEMODEL     → Change class MyEvent(m.BaseModel) to class MyEvent(m.DomainEvent)"
	echo "  SETATTR-HACK      → Remove setattr(obj, 'message_type', ...) — use self-describing handler"
	echo "  DIRECT-DISPATCHER → Use p.Dispatcher via DI, not FlextDispatcher() directly"
	echo "  QRY-BASEMODEL     → Change class MyQuery(m.BaseModel) to class MyQuery(m.Query)"
	exit 1
fi
