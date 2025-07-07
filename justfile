# Default recipe
default:
    @just --list

# Format code with rustfmt
format:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with fixes
lint-fix:
    cargo clippy --all-targets --all-features --fix

# Clean build artifacts
clean:
    cargo clean

# Build the project
build:
    cargo build --release

# Build with CLI feature
build-cli:
    cargo build --features cli

# Run the CLI with help
run-help:
    cargo run --features cli -- --help

# Run the CLI with arguments (use like: just run info)
run *args:
    cargo run --features cli -- {{args}}

# Run tests
test:
    cargo test

# Check code without building
check:
    cargo check

# Run examples
examples:
    @echo "🔧 Building examples..."
    @echo ""
    @echo "1. Simple Rust WASM:"
    just run compile --project ./examples/simple-rust --output ./examples/simple-rust/dist
    @echo ""
    @echo "2. Simple Web (wasm-bindgen):"
    just run compile --project ./examples/simple-web --output ./examples/simple-web/dist
    @echo ""
    @echo "3. Complex Yew App:"
    just run compile --project ./examples/complex-yew --output ./examples/complex-yew/dist

# Publish to crates.io (dry run first)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Install locally for testing
install:
    cargo install --path . --features cli

# Quick development cycle: format, lint, test, build
dev: format lint test build

# Full CI pipeline
ci: format lint test build
