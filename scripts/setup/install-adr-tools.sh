#!/bin/bash

# =============================================================================
# ADR Tools Installation Script - v0.2.1
# =============================================================================
#
# This script installs the professional ADR management tools required for
# the MCB project.
#
# Tools installed:
# - adrs: Professional ADR (Architecture Decision Records) management
#
# Requirements:
# - Rust toolchain installed
# - Cargo available in PATH
#
# Usage:
#   ./scripts/setup/install-adr-tools.sh
#
# =============================================================================

set -e # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo -e "${BLUE}🚀 Installing ADR Tools for MCB v0.2.1${NC}"
echo -e "${BLUE}====================================================${NC}"
echo ""

# Check if cargo is available
if ! command -v cargo &>/dev/null; then
	echo -e "${RED}❌ Error: Cargo is not installed or not in PATH${NC}"
	echo -e "${YELLOW}Please install Rust toolchain first: https://rustup.rs/${NC}"
	exit 1
fi

echo -e "${YELLOW}📦 Installing 'adrs' tool...${NC}"
if cargo install adrs; then
	echo -e "${GREEN}✅ 'adrs' tool installed successfully${NC}"
else
	echo -e "${RED}❌ Failed to install 'adrs' tool${NC}"
	exit 1
fi

# Verify installation
echo ""
echo -e "${YELLOW}🔍 Verifying installation...${NC}"
if ~/.cargo/bin/adrs --version &>/dev/null; then
	VERSION=$(~/.cargo/bin/adrs --version)
	echo -e "${GREEN}✅ ADR tools verified: ${VERSION}${NC}"
else
	echo -e "${RED}❌ ADR tools verification failed${NC}"
	exit 1
fi

# Check if .adr-dir configuration exists
echo ""
echo -e "${YELLOW}⚙️  Checking ADR configuration...${NC}"
if [ -f "${PROJECT_ROOT}/.adr-dir" ]; then
	ADR_DIR=$(cat "${PROJECT_ROOT}/.adr-dir")
	if [ -d "${PROJECT_ROOT}/${ADR_DIR}" ]; then
		echo -e "${GREEN}✅ ADR directory configured: ${ADR_DIR}${NC}"
	else
		echo -e "${RED}❌ ADR directory does not exist: ${ADR_DIR}${NC}"
		exit 1
	fi
else
	echo -e "${RED}❌ ADR configuration file not found: .adr-dir${NC}"
	exit 1
fi

# Test ADR functionality
echo ""
echo -e "${YELLOW}🧪 Testing ADR functionality...${NC}"
cd "${PROJECT_ROOT}"

if ~/.cargo/bin/adrs list &>/dev/null; then
	ADR_COUNT=$(~/.cargo/bin/adrs list | wc -l)
	echo -e "${GREEN}✅ ADR system functional: ${ADR_COUNT} ADRs found${NC}"
else
	echo -e "${RED}❌ ADR system test failed${NC}"
	exit 1
fi

echo ""
echo -e "${GREEN}🎉 ADR Tools Installation Complete!${NC}"
echo ""
echo -e "${BLUE}Available commands:${NC}"
echo -e "  make build WHAT=docs ACT=adr      - List ADRs, generate docs, show lifecycle status"
echo -e "  make build WHAT=docs ACT=adr-new  - Create new ADR"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Run 'make build WHAT=docs ACT=adr' to see existing ADRs"
echo -e "  2. Run 'make build WHAT=docs ACT=adr-new' to create your first ADR with the new tool"
echo -e "  3. The system will automatically maintain ADR compliance validation"
echo ""

exit 0
