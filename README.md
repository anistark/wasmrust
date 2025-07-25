# WasmRust

[![Crates.io Version](https://img.shields.io/crates/v/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads](https://img.shields.io/crates/d/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/wasmrust)](https://crates.io/crates/wasmrust) [![Open Source](https://img.shields.io/badge/open-source-brightgreen)](https://github.com/anistark/wasmrust) [![Contributors](https://img.shields.io/github/contributors/anistark/wasmrust)](https://github.com/anistark/wasmrust/graphs/contributors) ![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) WebAssembly plugin for [Wasmrun](https://github.com/anistark/wasmrun).
Compile and run Rust projects to WebAssembly to run easily on any wasm based ecosystem. 

## Installation

```sh
wasmrun plugin install wasmrust
```

### Standalone Installation (For testing only for now)

```sh
# For development and standalone usage
cargo install wasmrust --features cli
```

## 🚀 Usage

### Primary Usage (via Wasmrun)

```sh
# Wasmrun automatically detects Rust projects and uses wasmrust plugin

# Run Rust project (auto-detection)
wasmrun run ./my-rust-project

# Compile with optimization
wasmrun compile ./my-rust-project --optimization size

# Force Rust plugin usage
wasmrun run ./mixed-project --language rust

# Plugin-specific commands
wasmrun plugin info wasmrust
wasmrun plugin list
```

### Standalone Usage (CLI Mode)

```sh
# Direct wasmrust usage (when installed with --features cli)
wasmrust compile --project ./my-project --output ./dist
wasmrust run-for-execution ./my-project ./output
wasmrust info ./my-project
```

## 🔍 Project Detection & Support

WasmRust automatically detects and optimizes compilation for:

### Project Types
- **Standard WASM**: Basic Rust → WebAssembly compilation
- **wasm-bindgen**: JavaScript bindings with web-sys/js-sys integration
- **Web Applications**: Full-stack web apps with framework support

### Supported Frameworks
- **[Yew](https://yew.rs/)**: Modern Rust framework for creating multi-threaded frontend web apps
- **[Leptos](https://leptos.dev/)**: Full-stack, compile-time optimal Rust framework
- **[Dioxus](https://dioxuslabs.com/)**: Cross-platform GUI library for desktop, web, mobile
- **[Sycamore](https://sycamore-rs.netlify.app/)**: Reactive library for creating web apps
- **[Trunk](https://trunkrs.dev/)**: Build tool for Rust-generated WebAssembly web applications

### Build Strategies
- **Cargo**: Standard WASM compilation with `wasm32-unknown-unknown` target
- **wasm-pack**: Optimized builds for wasm-bindgen projects with JS integration
- **Trunk**: Complete web application builds with assets and bundling

## Examples

### 1. Simple Rust WASM

```rust
// examples/simple-rust/src/lib.rs
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 2. Web with wasm-bindgen

```rust
// examples/simple-web/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) {
    web_sys::console::log_1(&format!("Hello, {}!", name).into());
}
```

### 3. Yew Web Application
```rust
// examples/complex-yew/src/main.rs
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! { <h1>{"Hello Yew!"}</h1> }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

## Development

## Dev Installation (for dev testing and developement)

```sh
# Development build (fast compilation)
wasmrun compile ./project --optimization debug

# Production build (balanced)
wasmrun compile ./project --optimization release

# Size-optimized build (smallest output)
wasmrun compile ./project --optimization size
```

### Advanced Configuration
```sh
# Verbose output for debugging
wasmrun compile ./project --verbose

# Custom output directory
wasmrun compile ./project --output ./custom-dist

# Web application mode with live reload
wasmrun run ./web-project --watch --port 3000
```

## 🔧 Dependencies

### Required
- **cargo** - Rust build tool and package manager
- **rustc** - Rust compiler
- **wasm32-unknown-unknown** - WebAssembly compilation target

```sh
# Install target if missing
rustup target add wasm32-unknown-unknown
```

### Optional (Auto-detected)
- **wasm-pack** - Required for wasm-bindgen projects
- **trunk** - Required for web applications
- **wasm-opt** - WebAssembly optimizer (recommended)

### Dependency Checking
```sh
# Check your environment
wasmrun plugin info wasmrust

# Or with standalone CLI
wasmrust info ./my-project
```

## 🔄 Plugin Integration Details

### How It Works
1. **Project Detection**: Wasmrun scans for `Cargo.toml` and `.rs` files
2. **Plugin Loading**: WasmRust loads via dynamic library or binary execution
3. **Framework Analysis**: Automatic detection of web frameworks and build tools
4. **Optimized Compilation**: Framework-specific build strategies
5. **Asset Generation**: WASM + JS + HTML output as needed

### Compatibility
- **Unix/Linux/macOS**: Dynamic loading preferred
- **Windows**: Binary fallback mode
- **All platforms**: Graceful degradation ensures functionality

## 🔍 Troubleshooting

### Common Issues

**"Plugin not found"**:
```sh
# Verify installation
wasmrun plugin list
wasmrun plugin info wasmrust

# Reinstall if needed
wasmrun plugin install wasmrust
```

**"wasm32-unknown-unknown target not found"**:
```sh
rustup target add wasm32-unknown-unknown
```

**"wasm-pack not found" (for wasm-bindgen projects)**:
```sh
cargo install wasm-pack
```

**"trunk not found" (for web applications)**:
```sh
cargo install trunk
```

### Debug Mode
```sh
# Enable verbose output
wasmrun compile ./project --verbose

# Check dependencies
wasmrun plugin info wasmrust

# Test plugin loading
WASMRUN_VERBOSE=1 wasmrun run ./project
```

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests and documentation
5. Submit a pull request

## 📄 License

[MIT License](./LICENSE) - see the LICENSE file for details.

## 🔗 Related Projects

- **[Wasmrun](https://github.com/anistark/wasmrun)** - Universal WebAssembly runtime and plugin system
- **[WasmGo](https://github.com/anistark/wasmgo)** - Go WebAssembly plugin for Wasmrun
- **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)** - Facilitating high-level interactions between Wasm modules and JavaScript
- **[Trunk](https://github.com/thedodd/trunk)** - Build tool for Rust-generated WebAssembly

**Made with ❤️ for the Rust and WebAssembly communities**

*⭐ If you find WasmRust useful, please consider starring the repository!*
