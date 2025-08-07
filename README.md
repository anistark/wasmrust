# 🦀 WasmRust

[![Crates.io Version](https://img.shields.io/crates/v/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads](https://img.shields.io/crates/d/wasmrust)](https://crates.io/crates/wasmrust) [![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/wasmrust)](https://crates.io/crates/wasmrust) [![Open Source](https://img.shields.io/badge/open-source-brightgreen)](https://github.com/anistark/wasmrust) [![Contributors](https://img.shields.io/github/contributors/anistark/wasmrust)](https://github.com/anistark/wasmrust/graphs/contributors) ![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) **Rust to WebAssembly compiler plugin for [Wasmrun](https://github.com/anistark/wasmrun)**. Compile and run Rust projects to WebAssembly to run easily on any wasm based ecosystem.

## 📦 Installation

### Primary Installation (Wasmrun Plugin)

```sh
# Install wasmrun first
cargo install wasmrun

# Install the wasmrust plugin  
wasmrun plugin install wasmrust

# Verify installation
wasmrun plugin info wasmrust
```

### Standalone Installation (Development/Testing)

```sh
# Install as standalone CLI tool
cargo install wasmrust --features cli

# Verify standalone installation
wasmrust info
```

### Library Integration

```toml
[dependencies]
wasmrust = "0.2.1"

# For wasmrun plugin development
wasmrust = { version = "0.2.1", features = ["wasmrun-integration"] }

# For CLI usage
wasmrust = { version = "0.2.1", features = ["cli"] }
```

## 🛠️ Usage

### Primary Usage (via Wasmrun) - Recommended

Wasmrun automatically detects Rust projects and uses the wasmrust plugin:

```sh
# Automatic project detection and compilation
wasmrun ./my-rust-project

# Web application with live reload  
wasmrun ./my-yew-app --watch

# Compile with specific optimization
wasmrun compile ./my-project --optimization size

# Force Rust plugin usage (mixed projects)
wasmrun ./mixed-project --language rust

# Plugin management
wasmrun plugin info wasmrust
wasmrun plugin list
```

### Standalone Usage (Development/Testing)

For development, testing, or environments without wasmrun:

```sh
# Compile project to WebAssembly
wasmrust compile --project ./my-project --output ./dist

# Run project for execution (AOT compilation)
wasmrust run ./my-project

# Inspect project structure and dependencies
wasmrust inspect ./my-project

# Check if project is supported
wasmrust can-handle ./my-project

# Check system dependencies
wasmrust check-deps

# Clean build artifacts
wasmrust clean ./my-project

# Show supported frameworks
wasmrust frameworks
```

### Library Usage

```rust
use wasmrust::{WasmRustPlugin, CompileConfig, OptimizationLevel, TargetType};

let plugin = WasmRustPlugin::new();

// Check if project is supported
if plugin.can_handle("./my-project") {
    let config = CompileConfig {
        project_path: "./my-project".to_string(),
        output_dir: "./dist".to_string(),
        optimization: OptimizationLevel::Release,
        target_type: TargetType::WebApp,
        verbose: true,
    };
    
    match plugin.compile(&config) {
        Ok(result) => {
            println!("WASM: {}", result.wasm_path);
            if let Some(js_path) = result.js_path {
                println!("JS: {}", js_path);
            }
        }
        Err(e) => eprintln!("Compilation failed: {}", e),
    }
}
```

## 🎯 Supported Project Types & Frameworks

### Project Types (Auto-detected)

| Type | Description | Output | Build Tool |
|------|-------------|---------|------------|
| **Standard WASM** | Basic Rust → WebAssembly | `.wasm` file | `cargo` |
| **wasm-bindgen** | JavaScript integration | `.wasm` + `.js` | `wasm-pack` |
| **Web Application** | Full-stack web apps | Complete bundle | `trunk` / `wasm-pack` |

### Supported Web Frameworks

| Framework | Auto-Detection | Build Strategy | Status |
|-----------|----------------|----------------|---------|
| **[Yew](https://yew.rs/)** | `yew` dependency | trunk → wasm-pack | ✅ Full Support |
| **[Leptos](https://leptos.dev/)** | `leptos` dependency | trunk → wasm-pack | ✅ Full Support |
| **[Dioxus](https://dioxuslabs.com/)** | `dioxus` dependency | wasm-pack | ✅ Full Support |
| **[Sycamore](https://sycamore-rs.netlify.app/)** | `sycamore` dependency | wasm-pack | ✅ Full Support |
| **[Trunk](https://trunkrs.dev/)** | `Trunk.toml` present | trunk | ✅ Full Support |

### Framework Examples

#### Standard Rust WASM
```toml
[package]
name = "my-wasm-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
```

#### wasm-bindgen Project
```toml
[package]
name = "my-bindgen-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
```

#### Yew Web Application
```toml
[package]
name = "my-yew-app"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = "0.21"
wasm-bindgen = "0.2"
```

## 🔧 Build Strategies & Optimization

### Build Strategy Selection

WasmRust intelligently selects the optimal build strategy:

```
Project Analysis
      ↓
Framework Detection (Yew, Leptos, etc.)
      ↓
Build Tool Selection:
  • Standard WASM → cargo build
  • wasm-bindgen → wasm-pack  
  • Web Apps → trunk (preferred) → wasm-pack (fallback)
      ↓
Optimization Application
      ↓
Output Generation
```

### Optimization Levels

| Level | Compilation Time | File Size | Performance | Use Case |
|-------|------------------|-----------|-------------|----------|
| **debug** | Fast ⚡ | Large 📦 | Basic ⭐ | Development, debugging |
| **release** | Moderate ⏱️ | Medium 📦 | Good ⭐⭐⭐ | Production builds |
| **size** | Slow 🐌 | Minimal 📦 | Good ⭐⭐⭐ | Bandwidth-constrained |

### Advanced Optimization

```toml
# Cargo.toml optimization for smallest WASM
[profile.release]
opt-level = "s"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Slower compile, smaller binary
panic = "abort"          # Smaller binary
strip = "symbols"        # Remove debug symbols

[profile.release.package."*"]
opt-level = "s"

# Web-specific optimizations
[dependencies]
console_error_panic_hook = "0.1"
wee_alloc = "0.4"
```

## 🔍 Project Analysis & Dependencies

### Inspect Your Project

```sh
wasmrust inspect ./my-project
```

**Example Output:**
```
🔍 Analyzing Rust project...

📊 Project Analysis
═══════════════════
📁 Name: my-yew-app
🏷️  Version: 0.1.0  
🎯 Type: Web Application
🔧 Build Strategy: trunk + wasm-pack
🌐 Frameworks: yew, trunk

📋 Dependencies
═══════════════
Required:
   ✅ cargo - Rust build tool
   ✅ rustc - Rust compiler  
   ✅ wasm32-unknown-unknown - WebAssembly compilation target
   ✅ trunk - Required for web application builds

Optional:
   ✅ rustup - Rust toolchain manager
   ⚠️  wasm-opt - WebAssembly optimizer

🎉 Project is ready to build!
```

### System Dependencies

#### Required Tools
- **Rust Toolchain**: `rustup`, `cargo`, `rustc`
- **WASM Target**: `wasm32-unknown-unknown`

#### Optional Tools (Auto-detected)
- **wasm-pack**: For wasm-bindgen projects
- **trunk**: For web applications  
- **wasm-opt**: For additional optimization

#### Quick Installation

```sh
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install additional tools
cargo install wasm-pack trunk wasm-opt

# Verify installation
wasmrust check-deps
```

## 🔄 Live Development & Watch Mode

### File Watching (via Wasmrun)

WasmRust automatically monitors:
- `src/**/*.rs` - Source files
- `Cargo.toml` - Dependencies and configuration
- `Trunk.toml` - Trunk configuration  
- `assets/`, `static/`, `public/` - Static assets
- `style.css`, `index.html` - Web assets

### Development Workflow

```sh
# Start development server with live reload
wasmrun ./my-project --watch
```

## ⚙️ Configuration

### Project Configuration

Create `wasmrun.toml` in your project root:

```toml
[project]
language = "rust"

[build]
optimization = "release"
target_type = "webapp"
output_dir = "./dist"

[rust]
build_strategy = "trunk"        # cargo, wasm-pack, trunk
wasm_pack_target = "web"        # web, bundler, nodejs
enable_optimization = true
custom_flags = ["--features", "web"]
```

### Global Plugin Configuration

Configure in `~/.wasmrun/config.toml`:

```toml
[external_plugins.wasmrust]
enabled = true
auto_update = true
install_path = "/home/user/.wasmrun/plugins/wasmrust"

[external_plugins.wasmrust.defaults]
optimization = "size"
verbose = false
build_strategy = "auto"
```

### Environment Variables

```sh
# Enable verbose compilation
export WASMRUST_VERBOSE=1

# Custom optimization flags
export RUSTFLAGS="-C target-feature=+simd128"

# Force build strategy
export WASMRUST_BUILD_STRATEGY=trunk
```

## 🔧 Plugin Architecture & Integration

### Wasmrun Plugin Interface

WasmRust implements the full Wasmrun plugin architecture:

```rust
// Plugin trait implementation
impl Plugin for WasmrustPlugin {
    fn info(&self) -> &PluginInfo;
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn get_builder(&self) -> Box<dyn WasmBuilder>;
}

// Builder trait implementation  
impl WasmBuilder for WasmrustBuilder {
    fn build(&self, config: &BuildConfig) -> CompilationResult<BuildResult>;
    fn check_dependencies(&self) -> Vec<String>;
    fn validate_project(&self, project_path: &str) -> CompilationResult<()>;
    fn clean(&self, project_path: &str) -> Result<()>;
    // ... additional methods
}
```

### Dynamic Loading Support

WasmRust supports both library integration and dynamic loading:

```c
// C interface for dynamic loading
extern "C" {
    fn wasmrun_plugin_create() -> *mut c_void;
    fn wasmrust_can_handle_project(builder: *const c_void, path: *const c_char) -> bool;
    fn wasmrust_build(builder: *const c_void, config: *const BuildConfigC) -> *mut BuildResultC;
    // ... additional C functions
}
```

### Plugin Registration

```rust
// Rust integration
use wasmrust::create_plugin;

let plugin = create_plugin(); // Returns Box<dyn Plugin>

// C integration  
extern "C" fn wasmrun_plugin_create() -> *mut c_void;
```

## 🔍 Troubleshooting

### Common Issues

**"Plugin not found"**
```sh
# Verify plugin installation
wasmrun plugin list
wasmrun plugin info wasmrust

# Reinstall if needed
wasmrun plugin install wasmrust
```

**"wasm32-unknown-unknown target not found"**
```sh
rustup target add wasm32-unknown-unknown
```

**"wasm-pack not found" (for wasm-bindgen projects)**
```sh
cargo install wasm-pack
```

**"trunk not found" (for web applications)**
```sh
cargo install trunk
```

**"Compilation timeout"**
```sh
# Increase timeout for large projects
wasmrun compile ./large-project --timeout 300

# Use incremental compilation
export CARGO_INCREMENTAL=1
```

## 🧪 Testing & Development

### Running Tests

```sh
# Run all tests
cargo test

# Test wasmrun integration
cargo test --features wasmrun-integration

# Test CLI functionality
cargo test --features cli

# Integration tests (requires Rust toolchain)
cargo test test_actual_compilation -- --ignored
```

### Development Setup

TBD

### Contributing

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** with tests
4. **Run the test suite**: `cargo test --all-features`
5. **Update documentation** if needed
6. **Submit a pull request**

#### Adding Framework Support

1. **Update detection logic** in `detect_project_type_and_frameworks()`
2. **Add build strategy** in `determine_build_strategy()`  
3. **Implement compilation** in framework-specific methods
4. **Add tests** and update documentation
5. **Update README** with framework details

## 📊 Benchmarks & Performance

### Compilation Speed

| Project Type | Debug | Release | Size |
|-------------|-------|---------|------|
| Simple WASM | ~5s | ~15s | ~25s |
| wasm-bindgen | ~10s | ~30s | ~45s |
| Yew App | ~15s | ~45s | ~60s |

### Output Size Comparison

| Optimization | Simple WASM | wasm-bindgen | Yew App |
|-------------|-------------|--------------|---------|
| debug | ~500KB | ~800KB | ~1.2MB |
| release | ~200KB | ~400KB | ~600KB |
| size | ~100KB | ~250KB | ~400KB |

*Benchmarks on Apple M1, Rust 1.70, realistic projects*

## 🔗 Related Projects & Ecosystem

### Core Dependencies
- **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)** - JavaScript integration
- **[wasm-pack](https://github.com/rustwasm/wasm-pack)** - WebAssembly toolkit
- **[trunk](https://github.com/thedodd/trunk)** - Web application bundler

### Web Frameworks
- **[Yew](https://github.com/yewstack/yew)** - Modern Rust web framework
- **[Leptos](https://github.com/leptos-rs/leptos)** - Full-stack Rust framework
- **[Dioxus](https://github.com/dioxuslabs/dioxus)** - Cross-platform GUI library
- **[Sycamore](https://github.com/sycamore-rs/sycamore)** - Reactive web library

### Related Tools
- **[Wasmrun](https://github.com/anistark/wasmrun)** - Universal WebAssembly runtime
- **[WasmGo](https://github.com/anistark/wasmgo)** - Go WebAssembly plugin
- **[binaryen](https://github.com/WebAssembly/binaryen)** - WebAssembly optimizer

## 📄 License

[MIT License](./LICENSE) - see the LICENSE file for details.

## 🤝 Contributing & Community

### Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/anistark/wasmrust/issues)
- **Discussions**: [Community discussions](https://github.com/anistark/wasmrust/discussions)
- **Wasmrun Discord**: [Join the community](https://discord.gg/wasmrun)

### Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for:
- Code contribution guidelines
- Development setup instructions
- Testing requirements
- Documentation standards

**Made with ❤️ for the Rust and WebAssembly communities**

*⭐ If you find WasmRust useful, please consider starring the repository!*
