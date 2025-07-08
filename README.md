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
- **trunk**: For web applications

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

```bash
# Build and test examples
just examples

# Format, lint, test, build
just dev

# Clean outputs
just clean-examples
```

## Dependencies

### Required
- `cargo` (Rust toolchain)
- `rustc` (Rust compiler)  
- `wasm32-unknown-unknown` target

### Optional (auto-detected)
- `wasm-pack` (for wasm-bindgen projects)
- `trunk` (for web applications)

Check your setup:
```bash
wasmrust check
```

## License

MIT
