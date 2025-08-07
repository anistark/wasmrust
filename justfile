# WasmRust Development Justfile
# Rust to WebAssembly compiler plugin for Wasmrun

# Default recipe - show available commands
default:
    @echo "🦀 WasmRust Development Commands"
    @echo "==============================="
    @just --list

# ============================================================================
# DEVELOPMENT COMMANDS
# ============================================================================

# Format code with rustfmt
format:
    @echo "🎨 Formatting code..."
    cargo fmt

# Run clippy linter
lint:
    @echo "🔍 Running linter..."
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with automatic fixes
lint-fix:
    @echo "🔧 Running linter with fixes..."
    cargo clippy --all-targets --all-features --fix

# Check code without building
check:
    @echo "✅ Checking code..."
    cargo check --all-features

# Check code with specific features
check-features feature:
    @echo "✅ Checking with feature: {{feature}}"
    cargo check --features {{feature}}

# ============================================================================
# BUILD COMMANDS
# ============================================================================

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Build the project (library mode)
build: format lint test
    @echo "🔨 Building library..."
    cargo build --release

# Build with CLI feature
build-cli:
    @echo "🔨 Building with CLI..."
    cargo build --release --features cli

# Build with wasmrun integration. Experimental feature.
build-wasmrun:
    @echo "🔨 Building with wasmrun integration..."
    cargo build --release --features wasmrun-integration

# Build all feature combinations
build-all:
    @echo "🔨 Building all feature combinations..."
    @echo "1. Library only (no features):"
    cargo build --release --no-default-features
    @echo "2. CLI only:"
    cargo build --release --no-default-features --features cli
    @echo "3. Wasmrun integration only:"
    cargo build --release --no-default-features --features wasmrun-integration
    @echo "4. All features:"
    cargo build --release --all-features

# ============================================================================
# TESTING COMMANDS
# ============================================================================

# Run all tests
test:
    @echo "🧪 Running tests..."
    cargo test

# Run tests with specific features
test-features feature:
    @echo "🧪 Running tests with feature: {{feature}}"
    cargo test --features {{feature}}

# Run tests with wasmrun integration
test-wasmrun:
    @echo "🧪 Running wasmrun integration tests..."
    cargo test --features wasmrun-integration

# Run integration tests (requires Rust toolchain)
test-integration:
    @echo "🧪 Running integration tests..."
    cargo test test_actual_compilation -- --ignored

# Run all tests with coverage
test-coverage:
    @echo "🧪 Running tests with coverage..."
    cargo test --all-features

# Test specific module
test-module module:
    @echo "🧪 Testing module: {{module}}"
    cargo test {{module}}

# ============================================================================
# CLI COMMANDS
# ============================================================================

# Run the CLI with help
cli-help:
    @echo "📖 Showing CLI help..."
    cargo run --features cli -- --help

# Run CLI info command
cli-info:
    @echo "ℹ️  CLI info..."
    cargo run --features cli -- info

# Run CLI frameworks command
cli-frameworks:
    @echo "🌐 Showing supported frameworks..."
    cargo run --features cli -- frameworks

# Check CLI dependencies
cli-check-deps:
    @echo "🔍 Checking CLI dependencies..."
    cargo run --features cli -- check-deps

# Run CLI with custom arguments
cli *args:
    @echo "🚀 Running CLI with args: {{args}}"
    cargo run --features cli -- {{args}}

# ============================================================================
# EXAMPLE MANAGEMENT
# ============================================================================

# Set up example projects
setup-examples:
    @echo "📁 Setting up example projects..."
    @mkdir -p examples/{simple-rust,simple-web,complex-yew,trunk-app}/src
    @echo "✅ Example directories created"

# Create simple Rust WASM example
create-simple-rust:
    @echo "📝 Creating simple Rust WASM example..."
    @mkdir -p examples/simple-rust/src
    @echo '[package]\nname = "simple-rust"\nversion = "0.1.0"\nedition = "2021"\n\n[lib]\ncrate-type = ["cdylib"]' > examples/simple-rust/Cargo.toml
    @echo '#[no_mangle]\npub extern "C" fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[no_mangle]\npub extern "C" fn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }\n}' > examples/simple-rust/src/lib.rs

# Create wasm-bindgen example
create-simple-web:
    @echo "📝 Creating wasm-bindgen example..."
    @mkdir -p examples/simple-web/src
    @echo '[package]\nname = "simple-web"\nversion = "0.1.0"\nedition = "2021"\n\n[lib]\ncrate-type = ["cdylib"]\n\n[dependencies]\nwasm-bindgen = "0.2"\nweb-sys = "0.3"\njs-sys = "0.3"' > examples/simple-web/Cargo.toml
    @echo 'use wasm_bindgen::prelude::*;\n\n#[wasm_bindgen]\nextern "C" {\n    fn alert(s: &str);\n    #[wasm_bindgen(js_namespace = console)]\n    fn log(s: &str);\n}\n\n#[wasm_bindgen]\npub fn greet(name: &str) {\n    log(&format!("Hello from Rust, {}!", name));\n}\n\n#[wasm_bindgen]\npub struct Calculator;\n\n#[wasm_bindgen]\nimpl Calculator {\n    #[wasm_bindgen(constructor)]\n    pub fn new() -> Calculator {\n        Calculator\n    }\n    \n    #[wasm_bindgen]\n    pub fn add(&self, a: f64, b: f64) -> f64 {\n        a + b\n    }\n}' > examples/simple-web/src/lib.rs

# Create Yew example
create-complex-yew:
    @echo "📝 Creating Yew example..."
    @mkdir -p examples/complex-yew/src
    @echo '[package]\nname = "complex-yew"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nyew = { version = "0.21", features = ["csr"] }\nwasm-bindgen = "0.2"\nweb-sys = "0.3"' > examples/complex-yew/Cargo.toml
    @echo 'use yew::prelude::*;\n\n#[function_component(Counter)]\nfn counter() -> Html {\n    let count = use_state(|| 0);\n    let onclick = {\n        let count = count.clone();\n        move |_| count.set(*count + 1)\n    };\n\n    html! {\n        <div>\n            <h2>{ "Counter: " }{ *count }</h2>\n            <button {onclick}>{ "Increment" }</button>\n        </div>\n    }\n}\n\n#[function_component(App)]\nfn app() -> Html {\n    html! {\n        <div>\n            <h1>{ "Hello Yew!" }</h1>\n            <Counter />\n        </div>\n    }\n}\n\nfn main() {\n    yew::Renderer::<App>::new().render();\n}' > examples/complex-yew/src/main.rs
    @echo '<!DOCTYPE html>\n<html>\n<head>\n    <meta charset="utf-8">\n    <title>Yew App</title>\n</head>\n<body></body>\n</html>' > examples/complex-yew/index.html

# Create Trunk example
create-trunk-app:
    @echo "📝 Creating Trunk example..."
    @mkdir -p examples/trunk-app/src
    @echo '[package]\nname = "trunk-app"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nyew = { version = "0.21", features = ["csr"] }\nwasm-bindgen = "0.2"' > examples/trunk-app/Cargo.toml
    @echo '[build]\ntarget = "index.html"\n\n[watch]\nwatch = ["src", "Cargo.toml"]\nignore = ["dist"]' > examples/trunk-app/Trunk.toml
    @echo 'use yew::prelude::*;\n\n#[function_component(App)]\nfn app() -> Html {\n    html! {\n        <div>\n            <h1>{ "Trunk App" }</h1>\n            <p>{ "Built with Trunk!" }</p>\n        </div>\n    }\n}\n\nfn main() {\n    yew::Renderer::<App>::new().render();\n}' > examples/trunk-app/src/main.rs
    @echo '<!DOCTYPE html>\n<html>\n<head>\n    <meta charset="utf-8">\n    <title>Trunk App</title>\n</head>\n<body></body>\n</html>' > examples/trunk-app/index.html

# Create all examples
create-examples: create-simple-rust create-simple-web create-complex-yew create-trunk-app
    @echo "✅ All examples created!"

# ============================================================================
# EXAMPLE TESTING
# ============================================================================

# Check if examples can be handled
check-examples:
    @echo "🔍 Checking if examples can be handled..."
    @echo ""
    @if [ -d "examples/simple-rust" ]; then \
        echo "1. Checking simple-rust:"; \
        just cli can-handle examples/simple-rust; \
        echo ""; \
    fi
    @if [ -d "examples/simple-web" ]; then \
        echo "2. Checking simple-web:"; \
        just cli can-handle examples/simple-web; \
        echo ""; \
    fi
    @if [ -d "examples/complex-yew" ]; then \
        echo "3. Checking complex-yew:"; \
        just cli can-handle examples/complex-yew; \
        echo ""; \
    fi
    @if [ -d "examples/trunk-app" ]; then \
        echo "4. Checking trunk-app:"; \
        just cli can-handle examples/trunk-app; \
    fi

# Inspect example projects
inspect-examples:
    @echo "📊 Inspecting example projects..."
    @echo ""
    @if [ -d "examples/simple-rust" ]; then \
        echo "1. Inspecting simple-rust:"; \
        just cli inspect examples/simple-rust; \
        echo ""; \
    fi
    @if [ -d "examples/simple-web" ]; then \
        echo "2. Inspecting simple-web:"; \
        just cli inspect examples/simple-web; \
        echo ""; \
    fi
    @if [ -d "examples/complex-yew" ]; then \
        echo "3. Inspecting complex-yew:"; \
        just cli inspect examples/complex-yew; \
        echo ""; \
    fi

# Compile examples
compile-examples:
    @echo "🔨 Compiling examples..."
    @echo ""
    @if [ -d "examples/simple-rust" ]; then \
        echo "1. Compiling simple-rust:"; \
        just cli compile --project examples/simple-rust --output examples/simple-rust/dist; \
        echo ""; \
    fi
    @if [ -d "examples/simple-web" ]; then \
        echo "2. Compiling simple-web:"; \
        just cli compile --project examples/simple-web --output examples/simple-web/dist; \
        echo ""; \
    fi
    @if [ -d "examples/complex-yew" ]; then \
        echo "3. Compiling complex-yew:"; \
        just cli compile --project examples/complex-yew --output examples/complex-yew/dist; \
        echo ""; \
    fi

# Compile examples with verbose output
compile-examples-verbose:
    @echo "🔨 Compiling examples (verbose)..."
    @echo ""
    @if [ -d "examples/simple-rust" ]; then \
        echo "1. Compiling simple-rust (verbose):"; \
        just cli compile --project examples/simple-rust --output examples/simple-rust/dist --verbose; \
        echo ""; \
    fi
    @if [ -d "examples/simple-web" ]; then \
        echo "2. Compiling simple-web (verbose):"; \
        just cli compile --project examples/simple-web --output examples/simple-web/dist --verbose; \
        echo ""; \
    fi
    @if [ -d "examples/complex-yew" ]; then \
        echo "3. Compiling complex-yew (verbose):"; \
        just cli compile --project examples/complex-yew --output examples/complex-yew/dist --verbose; \
        echo ""; \
    fi

# Test specific example
test-example example:
    @echo "🧪 Testing {{example}} example..."
    @if [ -d "examples/{{example}}" ]; then \
        echo "✅ Directory exists"; \
        just cli can-handle examples/{{example}}; \
        echo ""; \
        just cli inspect examples/{{example}}; \
        echo ""; \
        just cli compile --project examples/{{example}} --output examples/{{example}}/dist --verbose; \
    else \
        echo "❌ Example '{{example}}' not found. Available examples:"; \
        ls examples/ 2>/dev/null || echo "No examples directory found. Run 'just create-examples' first."; \
    fi

# Clean example outputs
clean-examples:
    @echo "🧹 Cleaning example outputs..."
    @rm -rf examples/*/dist
    @rm -rf examples/*/target
    @echo "✅ Example outputs cleaned"

# ============================================================================
# INSTALLATION & PUBLISHING
# ============================================================================

# Install locally for testing (CLI mode)
install-cli:
    @echo "📦 Installing CLI locally..."
    cargo install --path . --features cli --force

# Install locally for testing (library mode)  
install-lib:
    @echo "📦 Installing library locally..."
    cargo install --path . --no-default-features --force

# Uninstall local installation
uninstall:
    @echo "🗑️  Uninstalling local installation..."
    cargo uninstall wasmrust

# Publish to crates.io (dry run first)
publish-test:
    @echo "🚀 Dry run publish to crates.io..."
    cargo publish --dry-run

# Publish to crates.io
publish:
    @echo "🚀 Publishing to crates.io..."
    cargo publish

# ============================================================================
# DEVELOPMENT WORKFLOWS
# ============================================================================

# Quick development cycle: format, lint, test, build
dev: format lint test build
    @echo "✅ Development cycle complete!"

# Full development cycle with all features
dev-full: format lint test-wasmrun build-all
    @echo "✅ Full development cycle complete!"

# Prepare for release: full test, build all, test publish
release-prep: format lint test-coverage build-all publish-test
    @echo "✅ Release preparation complete!"

# Complete example workflow: create, test, compile
examples-full: create-examples check-examples inspect-examples compile-examples
    @echo "✅ Complete example workflow finished!"

# ============================================================================
# MAINTENANCE COMMANDS
# ============================================================================

# Update dependencies
update-deps:
    @echo "📦 Updating dependencies..."
    cargo update

# Check for outdated dependencies
check-outdated:
    @echo "🔍 Checking for outdated dependencies..."
    cargo outdated

# Generate documentation
docs:
    @echo "📚 Generating documentation..."
    cargo doc --all-features --no-deps --open

# Check security advisories
security-audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Run benchmarks (if available)
bench:
    @echo "⚡ Running benchmarks..."
    cargo bench

# ============================================================================
# UTILITY COMMANDS
# ============================================================================

# Show project information
info:
    @echo "🦀 WasmRust Project Information"
    @echo "==============================="
    @echo "Version: $(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')"
    @echo "Features: cli, wasmrun-integration"
    @echo "Examples: $(ls examples/ 2>/dev/null | wc -l || echo 0) available"
    @echo "Tests: $(cargo test --dry-run 2>&1 | grep -c 'test result:' || echo 'unknown')"

# Show dependency tree
deps-tree:
    @echo "🌳 Dependency tree..."
    cargo tree

# Show feature flags
features:
    @echo "🏴 Available features..."
    @echo "• cli              - Command line interface"
    @echo "• wasmrun-integration - Wasmrun plugin integration"
    @echo "• default          - wasmrun-integration (enabled by default)"

# Check workspace
workspace-check:
    @echo "🔍 Workspace check..."
    @echo "Current directory: $(pwd)"
    @echo "Cargo.toml exists: $(test -f Cargo.toml && echo 'Yes' || echo 'No')"
    @echo "src/ directory exists: $(test -d src && echo 'Yes' || echo 'No')"
    @echo "Examples directory: $(test -d examples && echo 'Yes' || echo 'No')"

# Show all available recipes with descriptions
help:
    @echo "🦀 WasmRust Development Commands"
    @echo "==============================="
    @echo ""
    @echo "📖 DEVELOPMENT:"
    @echo "  format           - Format code with rustfmt"
    @echo "  lint             - Run clippy linter"
    @echo "  lint-fix         - Run clippy with fixes"
    @echo "  check            - Check code without building"
    @echo ""
    @echo "🔨 BUILD:"
    @echo "  build            - Build library"
    @echo "  build-cli        - Build with CLI feature"
    @echo "  build-wasmrun    - Build with wasmrun integration"
    @echo "  build-all        - Build all feature combinations"
    @echo ""
    @echo "🧪 TESTING:"
    @echo "  test             - Run all tests"
    @echo "  test-wasmrun     - Run wasmrun integration tests"
    @echo "  test-integration - Run integration tests"
    @echo ""
    @echo "🚀 CLI:"
    @echo "  cli-help         - Show CLI help"
    @echo "  cli-info         - Show plugin info"
    @echo "  cli *args        - Run CLI with custom arguments"
    @echo ""
    @echo "📁 EXAMPLES:"
    @echo "  create-examples  - Create all example projects"
    @echo "  check-examples   - Check example compatibility"
    @echo "  compile-examples - Compile all examples"
    @echo "  test-example     - Test specific example"
    @echo ""
    @echo "🔧 WORKFLOWS:"
    @echo "  dev              - Quick development cycle"
    @echo "  dev-full         - Full development cycle"
    @echo "  examples-full    - Complete example workflow"
    @echo ""
    @echo "For more details: just --list"
