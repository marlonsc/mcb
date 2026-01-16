# Workspace-next Makefile
# For building the refactored crate structure

.PHONY: build test lint fmt clean check

# Build all crates
build:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace

# Run clippy lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Full check (build + lint + test)
check: build lint test
	@echo "All checks passed!"

# Check compilation only (fast)
verify:
	cargo check --workspace

# Build in release mode
release:
	cargo build --workspace --release
