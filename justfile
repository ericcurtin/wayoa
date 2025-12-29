# Wayoa Justfile - run `just` for available recipes

# Default recipe: show available recipes
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Check formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run all CI checks (build, test, lint, format check)
ci: build test lint fmt-check
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Run the compositor
run:
    cargo run

# Run in release mode
run-release:
    cargo run --release
