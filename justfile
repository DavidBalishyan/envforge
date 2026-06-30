# Show list of avilable commands
default:
    @just --list

# Build the project (debug)
build:
    cargo build

# Build and show the embedded git hash
build-verbose:
    cargo build 2>&1 && ./target/debug/envforge --version

# Run the release build
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Lint with clippy
lint:
    cargo clippy -- -D warnings

# Check the binary works
check:
    cargo build && ./target/debug/envforge doctor

# Run envforge doctor
doctor:
    cargo build && ./target/debug/envforge doctor

# Clean build artifacts
clean:
    cargo clean

# Full CI check (build + test + lint)
ci: build test lint

# Show the embedded version
version:
    @cargo build -q 2>/dev/null
    @target/debug/envforge --version
