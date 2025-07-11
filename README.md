# WasmRust

[![Crates.io Version](https://img.shields.io/crates/v/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads](https://img.shields.io/crates/d/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/wasmrust)](https://crates.io/crates/wasmrust) [![Open Source](https://img.shields.io/badge/open-source-brightgreen)](https://github.com/anistark/wasmrust) [![Contributors](https://img.shields.io/github/contributors/anistark/wasmrust)](https://github.com/anistark/wasmrust/graphs/contributors) ![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) WebAssembly plugin for [Wasmrun](https://github.com/anistark/wasmrun).
Compile and run Rust projects to WebAssembly to run easily on any wasm based ecosystem. 

## Installation

```sh
wasmrun plugin install wasmrust
```

## Usage

### Commands (using wasmrun with wasmrust as plugin)

```sh
# Run project (default command)
wasmrun -p ./my-project
wasmrun  # same as above with current directory

# Compile project
wasmrun compile --project ./my-project --output ./dist

# Check project and dependencies
wasmrun check --project ./my-project

# Show info
wasmrun info
```

### Options

```sh
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

## Dev Installation (for dev testing and developement)

```sh
cargo install --path . --features cli
```

Using justfile:

```sh
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
```sh
wasmrust check
```

## License

[MIT](./LICENSE)
