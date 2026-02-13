#!/bin/bash
# =============================================================================
# CI Setup Script - Centralized dependency installation
# =============================================================================
set -e

# Detect OS
OS=$(uname -s)

# Install protobuf tooling (always required)
case "$OS" in
Linux)
	if ! command -v protoc &>/dev/null || [ ! -f /usr/include/google/protobuf/descriptor.proto ]; then
		sudo apt-get update -qq >/dev/null
		sudo apt-get install -y -qq --no-install-recommends protobuf-compiler libprotobuf-dev >/dev/null
	fi
	if command -v protoc &>/dev/null && [ ! -f /usr/include/google/protobuf/descriptor.proto ]; then
		echo "ERROR: protoc is installed but /usr/include/google/protobuf/descriptor.proto is missing." >&2
		echo "Install package 'libprotobuf-dev' before running CI checks." >&2
		exit 1
	fi
	export PROTOC_INCLUDE=/usr/include
	if [[ -n "${GITHUB_ENV:-}" ]]; then
		echo "PROTOC_INCLUDE=/usr/include" >>"$GITHUB_ENV"
	fi
	;;
Darwin)
	if ! command -v protoc &>/dev/null; then
		brew install protobuf -q
	fi
	;;
MINGW* | MSYS* | CYGWIN*)
	if ! command -v protoc &>/dev/null; then
		choco install protoc -y --no-progress
	fi
	;;
esac

if command -v protoc &>/dev/null; then
	export PROTOC
	PROTOC=$(command -v protoc)
	if [[ -n "${GITHUB_ENV:-}" ]]; then
		echo "PROTOC=$PROTOC" >>"$GITHUB_ENV"
	fi
fi

# Parse optional flags
while [[ $# -gt 0 ]]; do
	case $1 in
	--install-audit)
		if ! command -v cargo-audit &>/dev/null; then
			cargo install cargo-audit --locked --quiet
		fi
		shift
		;;
	--install-coverage)
		if ! command -v cargo-tarpaulin &>/dev/null; then
			cargo install cargo-tarpaulin --locked --quiet
		fi
		shift
		;;
	--install-diagrams)
		if ! command -v plantuml &>/dev/null; then
			case "$OS" in
			Linux)
				sudo apt-get install -y -qq --no-install-recommends plantuml >/dev/null
				;;
			Darwin)
				brew install plantuml -q
				;;
			MINGW* | MSYS* | CYGWIN*)
				echo "Warning: PlantUML installation on Windows not automated. Please install manually." >&2
				;;
			*)
				echo "Warning: PlantUML installation not supported on $OS" >&2
				;;
			esac
		fi
		shift
		;;
	*)
		echo "Error: Unknown option: $1" >&2
		exit 1
		;;
	esac
done
