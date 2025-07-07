# WasmRust

Rust WebAssembly plugin for Wasmrun. Compile and run Rust projects as WebAssembly with intelligent build strategy detection.

## Installation

```bash
cargo install --path . --features cli
```

## Usage

### Commands

```bash
# Run project (default command)
wasmrust run --project ./my-project
wasmrust  # same as above with current directory

# Compile project
wasmrust compile --project ./my-project --output ./dist

# Check project and dependencies
wasmrust check --project ./my-project

# Show info
wasmrust info
```

### Options

```bash
# Optimization levels
--optimization debug|release|size

# Output directory
--output ./custom-dist

# Verbose output
--verbose
```

## Project Types

WasmRust automatically detects and handles:

- **Standard WASM**: Basic Rust → WebAssembly compilation
- **wasm-bindgen**: JS bindings with web-sys/js-sys
- **Web Applications**: Full web apps (Yew, Leptos, Dioxus, etc.)

## Build Strategies

- **cargo**: Standard WASM compilation
- **wasm-pack**: For wasm-bindgen projects  
- **trunk**: For web applications with Trunk

## Development (justfile)

```bash
# Install just: cargo install just

# Format code
just format

# Lint code
just lint

# Run tests
just test

# Build project
just build

# Build with CLI features
just build-cli

# Run CLI with arguments
just cli --help
just cli check --project ./examples/simple-rust

# Build and test examples
just examples

# Publish to crates.io
just publish-dry  # dry run first
just publish

# Development cycle
just dev  # format + lint + test + build
```

## Examples

### 1. Simple Rust WASM

```rust
// examples/simple-rust/src/lib.rs
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```bash
wasmrust compile --project ./examples/simple-rust
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

```bash
wasmrust compile --project ./examples/simple-web
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

```bash
wasmrust compile --project ./examples/complex-yew
```

## Dependencies

### Required
- `cargo` (Rust toolchain)
- `rustc` (Rust compiler)  
- `wasm32-unknown-unknown` target

### Optional (auto-detected)
- `wasm-pack` (for wasm-bindgen projects)
- `trunk` (for web applications)
- `wasm-opt` (WebAssembly optimizer)

Check your setup:
```bash
wasmrust check
```

## License

MIT
