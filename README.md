# WasmRust Plugin

[![Crates.io](https://img.shields.io/crates/v/wasmrust)](https://crates.io/crates/wasmrust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust WebAssembly plugin for [Wasmrun](https://github.com/anistark/wasmrun) - compile Rust projects to WebAssembly with wasm-bindgen and web framework support.

## Installation

### Via Wasmrun (Recommended)

```sh
wasmrun plugin install wasmrust
```

### Direct Installation (For Testing/Dev only)

```sh
cargo install wasmrust
```

### From Source (For contributors/experimental features)

```sh
git clone https://github.com/anistark/wasmrust
cd wasmrust
cargo install --path .
```

## Usage

### With Wasmrun

Once installed as a plugin, Wasmrun will automatically use wasmrust for Rust projects:

```sh
# Automatic detection for Rust projects
wasmrun run ./my-rust-project

# Compile Rust project to WASM
wasmrun compile ./my-rust-project --optimization size

# Web application development with live reload
wasmrun run ./my-yew-app --watch
```

### Standalone Usage (For testing/dev)

TBD

## Build Configuration

### Optimization Levels

- **`debug`** - Fast compilation, full symbols, no optimization
- **`release`** - Optimized for performance (default)
- **`size`** - Optimized for minimal file size

### Target Types

- **`wasm`** - Standard WebAssembly output
- **`webapp`** - Complete web application with HTML/JS/CSS

## Integration with Wasmrun

When installed as a Wasmrun plugin, wasmrust provides:

### Automatic Detection

Wasmrust automatically handles Rust projects when:
- `Cargo.toml` file is present in the project root
- `.rs` files are detected in the project

### Plugin Capabilities

```toml
[package.metadata.wasm-plugin.capabilities]
compile_wasm = true
compile_webapp = true
live_reload = true
optimization = true
custom_targets = ["wasm32-unknown-unknown", "web"]
```

### Dependency Checking

The plugin automatically verifies that required tools are installed and provides helpful error messages for missing dependencies.
