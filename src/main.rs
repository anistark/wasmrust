#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use wasmrust::{CompileConfig, OptimizationLevel, TargetType, WasmRustPlugin};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Run a Rust WebAssembly project for execution (default command)
    #[command(alias = "r")]
    Run {
        /// Project path containing Cargo.toml
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,

        /// Output directory for compiled files
        #[arg(short, long, default_value = "./dist", value_name = "DIR")]
        output: String,

        /// Optimization level for compilation
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,

        /// Enable verbose compilation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compile a Rust project to WebAssembly
    #[command(alias = "c")]
    Compile {
        /// Project path containing Cargo.toml
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,

        /// Output directory for compiled files
        #[arg(short, long, default_value = "./dist", value_name = "DIR")]
        output: String,

        /// Optimization level for compilation
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,

        /// Target type for compilation
        #[arg(long, value_enum, default_value = "wasm")]
        target: CliTarget,

        /// Enable verbose compilation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Inspect project structure, dependencies, and frameworks
    #[command(alias = "check")]
    Inspect {
        /// Project path to inspect
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,
    },

    /// Check if wasmrust can handle the project
    CanHandle {
        /// Project path to check
        #[arg(value_name = "PATH")]
        project: String,
    },

    /// Check dependencies and system requirements
    CheckDeps,

    /// Clean build artifacts
    Clean {
        /// Project path to clean
        #[arg(value_name = "PATH")]
        project: String,
    },

    /// Show plugin information and capabilities
    Info,

    /// Show supported frameworks and project types
    Frameworks,
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliOptimization {
    /// Fast compilation with debug symbols
    Debug,
    /// Balanced optimization for production
    Release,
    /// Smallest possible output size
    Size,
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliTarget {
    /// Standard WebAssembly module
    Wasm,
    /// Complete web application bundle
    WebApp,
}

#[cfg(feature = "cli")]
impl From<CliOptimization> for OptimizationLevel {
    fn from(opt: CliOptimization) -> Self {
        match opt {
            CliOptimization::Debug => OptimizationLevel::Debug,
            CliOptimization::Release => OptimizationLevel::Release,
            CliOptimization::Size => OptimizationLevel::Size,
        }
    }
}

#[cfg(feature = "cli")]
impl From<CliTarget> for TargetType {
    fn from(target: CliTarget) -> Self {
        match target {
            CliTarget::Wasm => TargetType::Wasm,
            CliTarget::WebApp => TargetType::WebApp,
        }
    }
}

#[cfg(feature = "cli")]
fn print_header() {
    println!(
        "🦀 {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("   {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
}

#[cfg(feature = "cli")]
fn check_project_validity(plugin: &WasmRustPlugin, project: &str) -> bool {
    if !plugin.can_handle(project) {
        eprintln!("❌ Error: Not a valid Rust project");
        eprintln!("   Looking for Cargo.toml in: {project}");
        eprintln!("   Make sure you're in a Rust project directory");
        return false;
    }
    true
}

#[cfg(feature = "cli")]
fn check_dependencies(plugin: &WasmRustPlugin) -> bool {
    let missing_deps = plugin.check_dependencies();
    if !missing_deps.is_empty() {
        eprintln!("❌ Missing required dependencies:");
        for dep in &missing_deps {
            eprintln!("   • {dep}");
        }
        eprintln!();
        eprintln!("💡 Installation suggestions:");
        if missing_deps
            .iter()
            .any(|d| d.contains("cargo") || d.contains("rustc"))
        {
            eprintln!("   • Install Rust: https://rustup.rs/");
        }
        if missing_deps
            .iter()
            .any(|d| d.contains("wasm32-unknown-unknown"))
        {
            eprintln!("   • Add WASM target: rustup target add wasm32-unknown-unknown");
        }
        if missing_deps.iter().any(|d| d.contains("wasm-pack")) {
            eprintln!("   • Install wasm-pack: cargo install wasm-pack");
        }
        return false;
    }
    true
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let plugin = WasmRustPlugin::new();

    // Default to Run command if no subcommand is provided
    let command = cli.command.unwrap_or(Commands::Run {
        project: ".".to_string(),
        output: "./dist".to_string(),
        optimization: CliOptimization::Release,
        verbose: false,
    });

    match command {
        Commands::Run {
            project,
            output,
            optimization,
            verbose,
        } => {
            if verbose {
                print_header();
                println!("🚀 Preparing Rust project for execution...");
                println!("📁 Project: {project}");
                println!("📦 Output: {output}");
                println!("🎯 Optimization: {optimization:?}");
                println!();
            }

            if !check_project_validity(&plugin, &project) {
                std::process::exit(1);
            }

            if !check_dependencies(&plugin) {
                std::process::exit(1);
            }

            match plugin.compile_for_aot_with_optimization(&project, &output, optimization.into()) {
                Ok(entry_point) => {
                    if verbose {
                        println!("✅ Project ready for execution!");
                        println!("🎯 Entry point: {entry_point}");
                    } else {
                        // For scripting - just output the entry point
                        println!("{entry_point}");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to prepare project for execution: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Compile {
            project,
            output,
            optimization,
            target,
            verbose,
        } => {
            if verbose {
                print_header();
                println!("🔨 Compiling Rust project to WebAssembly...");
                println!("📁 Project: {project}");
                println!("📦 Output: {output}");
                println!("🎯 Optimization: {optimization:?}");
                println!("🏗️  Target: {target:?}");
                println!();
            }

            if !check_project_validity(&plugin, &project) {
                std::process::exit(1);
            }

            if !check_dependencies(&plugin) {
                std::process::exit(1);
            }

            let config = CompileConfig {
                project_path: project.clone(),
                output_dir: output,
                optimization: optimization.into(),
                target_type: target.into(),
                verbose,
            };

            match plugin.compile(&config) {
                Ok(result) => {
                    println!("✅ Compilation completed successfully!");
                    println!("🎯 WASM file: {}", result.wasm_path);

                    if let Some(js_path) = result.js_path {
                        println!("📄 JS bindings: {js_path}");
                    }

                    if result.is_webapp {
                        println!("🌐 Web application bundle created");
                    }

                    if !result.additional_files.is_empty() {
                        println!("📂 Additional files: {}", result.additional_files.len());
                        if verbose {
                            for file in result.additional_files {
                                println!("   • {file}");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Compilation failed: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Inspect { project } => {
            print_header();
            println!("🔍 Inspecting Rust project...");
            println!();

            match plugin.inspect_project(&project) {
                Ok(info) => {
                    println!("📊 Project Analysis");
                    println!("═══════════════════");
                    println!("📁 Name: {}", info.name);
                    println!("🏷️  Version: {}", info.version);

                    let project_type_desc = match info.project_type {
                        wasmrust::ProjectType::StandardWasm => "Standard WebAssembly",
                        wasmrust::ProjectType::WasmBindgen => "WebAssembly with JS bindings",
                        wasmrust::ProjectType::WebApplication => "Web Application",
                    };
                    println!("🎯 Type: {project_type_desc}");

                    let strategy_desc = match info.build_strategy {
                        wasmrust::BuildStrategy::Cargo => "cargo build",
                        wasmrust::BuildStrategy::WasmPack => "wasm-pack",
                        wasmrust::BuildStrategy::Trunk => "trunk + wasm-pack",
                    };
                    println!("🔧 Build Strategy: {strategy_desc}");

                    if !info.frameworks.is_empty() {
                        println!("🌐 Frameworks: {}", info.frameworks.join(", "));
                    }

                    println!();
                    println!("📋 Dependencies");
                    println!("═══════════════");

                    let mut all_good = true;

                    println!("Required:");
                    for dep in &info.dependencies.required {
                        let status = if dep.available { "✅" } else { "❌" };
                        println!("   {} {} - {}", status, dep.name, dep.reason);
                        if !dep.available {
                            all_good = false;
                        }
                    }

                    if !info.dependencies.optional.is_empty() {
                        println!();
                        println!("Optional:");
                        for dep in &info.dependencies.optional {
                            let status = if dep.available { "✅" } else { "⚠️ " };
                            println!("   {} {} - {}", status, dep.name, dep.reason);
                        }
                    }

                    println!();
                    if all_good {
                        println!("🎉 Project is ready to build!");
                    } else {
                        println!(
                            "⚠️  Some required dependencies are missing. Install them to proceed."
                        );
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    match e {
                        wasmrust::WasmRustError::InvalidProject(msg) => {
                            eprintln!("❌ Invalid project: {msg}");
                        }
                        wasmrust::WasmRustError::TomlParse(parse_err) => {
                            eprintln!("❌ Invalid Cargo.toml syntax:");
                            eprintln!("   {parse_err}");
                        }
                        _ => {
                            eprintln!("❌ Error inspecting project: {e}");
                        }
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::CanHandle { project } => {
            if plugin.can_handle(&project) {
                println!("✅ Yes, wasmrust can handle this project");
                println!("📁 Found Cargo.toml at: {project}/Cargo.toml");

                // Additional project info
                if let Ok(info) = plugin.inspect_project(&project) {
                    println!("🎯 Project type: {:?}", info.project_type);
                    if !info.frameworks.is_empty() {
                        println!("🌐 Detected frameworks: {}", info.frameworks.join(", "));
                    }
                }
            } else {
                println!("❌ No, wasmrust cannot handle this project");
                println!("🔍 Looking for Cargo.toml in: {project}");
                std::process::exit(1);
            }
        }

        Commands::CheckDeps => {
            print_header();
            println!("🔍 Checking system dependencies...");
            println!();

            let missing = plugin.check_dependencies();

            if missing.is_empty() {
                println!("✅ All required dependencies are available!");

                // Show what we found
                println!();
                println!("📋 Available tools:");
                println!("   ✅ cargo - Rust build tool");
                println!("   ✅ rustc - Rust compiler");
                println!("   ✅ wasm32-unknown-unknown - WebAssembly target");

                // Check optional tools
                if plugin.is_tool_available("wasm-pack") {
                    println!("   ✅ wasm-pack - WebAssembly package tool");
                }
                if plugin.is_tool_available("trunk") {
                    println!("   ✅ trunk - Web application bundler");
                }
                if plugin.is_tool_available("wasm-opt") {
                    println!("   ✅ wasm-opt - WebAssembly optimizer");
                }
            } else {
                println!("❌ Missing required dependencies:");
                for dep in &missing {
                    println!("   • {dep}");
                }

                println!();
                println!("💡 Installation suggestions:");
                println!("   • Install Rust: https://rustup.rs/");
                println!("   • Add WASM target: rustup target add wasm32-unknown-unknown");
                println!("   • Install wasm-pack: cargo install wasm-pack");
                println!("   • Install trunk (for web apps): cargo install trunk");
                println!("   • Install wasm-opt: cargo install wasm-opt");

                std::process::exit(1);
            }
        }

        Commands::Clean { project } => {
            println!("🧹 Cleaning project artifacts: {project}");

            let output = std::process::Command::new("cargo")
                .args(["clean"])
                .current_dir(&project)
                .output()?;

            if output.status.success() {
                println!("✅ Project cleaned successfully!");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Clean failed: {stderr}");
                std::process::exit(1);
            }
        }

        Commands::Info => {
            print_header();
            println!("🔧 Plugin Information");
            println!("═════════════════════");
            println!("Name: {}", env!("CARGO_PKG_NAME"));
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));

            // Parse authors (they might be comma-separated)
            let authors = env!("CARGO_PKG_AUTHORS");
            if !authors.is_empty() {
                println!("Author(s): {authors}");
            }

            // Add repository if available
            if !env!("CARGO_PKG_REPOSITORY").is_empty() {
                println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
            }

            // Add homepage if available
            if !env!("CARGO_PKG_HOMEPAGE").is_empty() {
                println!("Homepage: {}", env!("CARGO_PKG_HOMEPAGE"));
            }

            // Add license if available
            if !env!("CARGO_PKG_LICENSE").is_empty() {
                println!("License: {}", env!("CARGO_PKG_LICENSE"));
            }

            println!();

            println!("🎯 Capabilities");
            println!("═══════════════");
            println!("✅ Standard WASM compilation");
            println!("✅ wasm-bindgen integration");
            println!("✅ Web application building");
            println!("✅ Live reload support");
            println!("✅ Multiple optimization levels");
            println!("✅ Framework auto-detection");
            println!();

            println!("📄 Usage");
            println!("════════");
            println!("Primary (via Wasmrun):");
            println!("   wasmrun run ./my-rust-project");
            println!("   wasmrun compile ./my-project --optimization size");
            println!();
            println!("Standalone (testing/development):");
            println!("   {} run ./my-project", env!("CARGO_PKG_NAME"));
            println!(
                "   {} compile ./my-project --target webapp",
                env!("CARGO_PKG_NAME")
            );
            println!("   {} inspect ./my-project", env!("CARGO_PKG_NAME"));
        }

        Commands::Frameworks => {
            print_header();
            println!("🌐 Supported Frameworks & Project Types");
            println!("═══════════════════════════════════════");
            println!();

            println!("📦 Project Types:");
            println!("   • Standard WASM    - Basic Rust → WebAssembly compilation");
            println!("   • wasm-bindgen     - JavaScript integration with web-sys/js-sys");
            println!("   • Web Applications - Full-stack web apps with asset bundling");
            println!();

            println!("🌐 Web Frameworks (Auto-detected):");
            println!("   • Yew              - Modern Rust framework for web apps");
            println!("   • Leptos           - Full-stack, compile-time optimal framework");
            println!("   • Dioxus           - Cross-platform GUI library");
            println!("   • Sycamore         - Reactive library for web apps");
            println!("   • Trunk            - Build tool for Rust WebAssembly apps");
            println!();

            println!("🔧 Build Tools:");
            println!("   • cargo            - Standard Rust build tool");
            println!("   • wasm-pack        - WebAssembly package tool");
            println!("   • trunk            - Web application bundler");
            println!();

            println!("🎯 Optimization Levels:");
            println!("   • debug            - Fast compilation, debug symbols");
            println!("   • release          - Balanced optimization");
            println!("   • size             - Smallest possible output");
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("❌ CLI feature not enabled");
    eprintln!();
    eprintln!("This binary was built without CLI support.");
    eprintln!("To use the command line interface:");
    eprintln!("   cargo install wasmrust --features cli");
    eprintln!();
    eprintln!("This library is primarily designed as a plugin for Wasmrun:");
    eprintln!("   wasmrun plugin install wasmrust");
    eprintln!("   wasmrun run ./my-rust-project");

    std::process::exit(1);
}
