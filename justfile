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

# Check individual example projects
check-examples:
    @echo "🔍 Checking examples..."
    @echo ""
    @echo "1. Checking simple-rust:"
    just run check --project ./examples/simple-rust
    @echo ""
    @echo "2. Checking simple-web:"
    just run check --project ./examples/simple-web
    @echo ""
    @echo "3. Checking complex-yew:"
    just run check --project ./examples/complex-yew

# Run examples with verbose output for debugging
examples-verbose:
    @echo "🔧 Building examples (verbose)..."
    @echo ""
    @echo "1. Simple Rust WASM:"
    just run compile --project ./examples/simple-rust --output ./examples/simple-rust/dist --verbose
    @echo ""
    @echo "2. Simple Web (wasm-bindgen):"
    just run compile --project ./examples/simple-web --output ./examples/simple-web/dist --verbose
    @echo ""
    @echo "3. Complex Yew App:"
    @echo "📄 Copying shared index.html template..."
    cp ./examples/index.html ./examples/complex-yew/index.html
    just run compile --project ./examples/complex-yew --output ./examples/complex-yew/dist --verbose

# Run examples with shared HTML template
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
    @echo "📄 Copying shared index.html template..."
    cp ./examples/index.html ./examples/complex-yew/index.html
    @echo "🔍 Verifying index.html was copied:"
    ls -la ./examples/complex-yew/index.html
    @echo "📋 Contents of index.html:"
    head -5 ./examples/complex-yew/index.html
    @echo "🔧 Building with verbose output:"
    just run compile --project ./examples/complex-yew --output ./examples/complex-yew/dist --verbose

# Clean example outputs
clean-examples:
    @echo "🧹 Cleaning example outputs..."
    rm -rf ./examples/*/dist
    rm -rf ./examples/*/target
    rm -f ./examples/complex-yew/index.html  # Remove copied template

# Test a specific example
test-example example:
    @echo "🧪 Testing {{example}} example..."
    just run check --project ./examples/{{example}}
    just run compile --project ./examples/{{example}} --output ./examples/{{example}}/dist --verbose

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
