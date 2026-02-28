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
		PROTOC_VERSION="${PROTOC_VERSION:-29.3}"
		PROTOC_SHA256="${PROTOC_SHA256:-57ea59e9f551ad8d71ffaa9b5cfbe0ca1f4e720972a1db7ec2d12ab44bff9383}"
		PROTOC_URL="https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-win64.zip"
		PROTOC_DIR="${TEMP:-/tmp}/protoc"
		PROTOC_ZIP="${TEMP:-/tmp}/protoc.zip"
		curl -sSfL "$PROTOC_URL" -o "$PROTOC_ZIP"
		if command -v sha256sum &>/dev/null; then
			ACTUAL_SHA256=$(sha256sum "$PROTOC_ZIP" | awk '{print $1}')
		elif command -v shasum &>/dev/null; then
			ACTUAL_SHA256=$(shasum -a 256 "$PROTOC_ZIP" | awk '{print $1}')
		else
			echo "ERROR: neither sha256sum nor shasum is available to verify protoc download." >&2
			exit 1
		fi
		if [[ "${ACTUAL_SHA256,,}" != "${PROTOC_SHA256,,}" ]]; then
			echo "ERROR: protoc checksum mismatch. expected=${PROTOC_SHA256} got=${ACTUAL_SHA256}" >&2
			exit 1
		fi
		unzip -qo "$PROTOC_ZIP" -d "$PROTOC_DIR"
		export PATH="$PROTOC_DIR/bin:$PATH"
		if [[ -n "${GITHUB_PATH:-}" ]]; then
			echo "$PROTOC_DIR/bin" >>"$GITHUB_PATH"
		fi
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

# Install ONNX Runtime (required by fastembed/ort for semantic embedding).
# ort-sys 2.0.0-rc.11 uses ORT_API_VERSION=23 which requires ONNX Runtime >= 1.23.x.
if [[ "$OS" == "Linux" ]] && ! ldconfig -p 2>/dev/null | grep -q 'libonnxruntime\.so'; then
	ORT_VERSION="1.23.2"
	ORT_ARCHIVE="onnxruntime-linux-x64-${ORT_VERSION}.tgz"
	ORT_URL="https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/${ORT_ARCHIVE}"
	ORT_TMP="/tmp/${ORT_ARCHIVE}"
	echo "Installing ONNX Runtime ${ORT_VERSION} for fastembed..." >&2
	curl -sSfL "$ORT_URL" -o "$ORT_TMP"
	tar -xzf "$ORT_TMP" -C /tmp
	sudo cp "/tmp/onnxruntime-linux-x64-${ORT_VERSION}/lib/libonnxruntime.so.${ORT_VERSION}" /usr/local/lib/
	sudo ln -sf "/usr/local/lib/libonnxruntime.so.${ORT_VERSION}" /usr/local/lib/libonnxruntime.so.1
	sudo ln -sf /usr/local/lib/libonnxruntime.so.1 /usr/local/lib/libonnxruntime.so
	sudo ldconfig
	rm -f "$ORT_TMP"
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
