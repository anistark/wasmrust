use clap::{Parser, Subcommand};
use wasmrust::{BuildConfig, OptimizationLevel, TargetType, WasmRustPlugin};

#[derive(Parser)]
#[command(name = "wasmrust")]
#[command(about = "Rust WebAssembly plugin for Wasmrun")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long, default_value = ".")]
        project: String,
        
        #[arg(short, long, default_value = "./dist")]
        output: String,
        
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,
        
        #[arg(long, value_enum, default_value = "wasm")]
        target: CliTarget,
        
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Compile for AOT execution (returns optimal entry point)
    Aot {
        #[arg(short, long, default_value = ".")]
        project: String,
        
        #[arg(short, long, default_value = "./dist")]
        output: String,
        
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,
        
        #[arg(short, long)]
        verbose: bool,
    },
    
    Check {
        #[arg(short, long, default_value = ".")]
        project: String,
    },
    
    Info,
}

#[derive(clap::ValueEnum, Clone)]
enum CliOptimization {
    Debug,
    Release,
    Size,
}

#[derive(clap::ValueEnum, Clone)]
enum CliTarget {
    Wasm,
    WebApp,
}

impl From<CliOptimization> for OptimizationLevel {
    fn from(opt: CliOptimization) -> Self {
        match opt {
            CliOptimization::Debug => OptimizationLevel::Debug,
            CliOptimization::Release => OptimizationLevel::Release,
            CliOptimization::Size => OptimizationLevel::Size,
        }
    }
}

impl From<CliTarget> for TargetType {
    fn from(target: CliTarget) -> Self {
        match target {
            CliTarget::Wasm => TargetType::Wasm,
            CliTarget::WebApp => TargetType::WebApp,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let plugin = WasmRustPlugin::new();

    match cli.command {
        Commands::Build {
            project,
            output,
            optimization,
            target,
            verbose,
        } => {
            let config = BuildConfig {
                project_path: project.clone(),
                output_dir: output,
                optimization: optimization.into(),
                target_type: target.into(),
                verbose,
            };

            if !plugin.can_handle(&project) {
                eprintln!("Error: Not a valid Rust project (no Cargo.toml found)");
                std::process::exit(1);
            }

            let missing_deps = plugin.check_dependencies();
            if !missing_deps.is_empty() {
                eprintln!("Error: Missing dependencies:");
                for dep in missing_deps {
                    eprintln!("  - {}", dep);
                }
                std::process::exit(1);
            }

            match plugin.build(&config) {
                Ok(result) => {
                    println!("✅ Build completed successfully!");
                    println!("📦 WASM: {}", result.wasm_path);
                    
                    if let Some(js_path) = result.js_path {
                        println!("📝 JS: {}", js_path);
                    }
                    
                    if result.is_webapp {
                        println!("🌐 Web application built successfully");
                    }

                    if !result.additional_files.is_empty() {
                        println!("📄 Additional files: {}", result.additional_files.len());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Build failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Aot {
            project,
            output,
            optimization,
            verbose,
        } => {
            if !plugin.can_handle(&project) {
                eprintln!("Error: Not a valid Rust project (no Cargo.toml found)");
                std::process::exit(1);
            }

            let missing_deps = plugin.check_dependencies();
            if !missing_deps.is_empty() {
                eprintln!("Error: Missing dependencies:");
                for dep in missing_deps {
                    eprintln!("  - {}", dep);
                }
                std::process::exit(1);
            }

            if verbose {
                println!("🔨 Compiling for AOT execution...");
            }

            match plugin.compile_for_execution(&project, &output) {
                Ok(entry_point) => {
                    if verbose {
                        println!("✅ AOT compilation successful!");
                        println!("📦 Entry point: {}", entry_point);
                    } else {
                        // For scripting - just output the entry point
                        println!("{}", entry_point);
                    }
                }
                Err(e) => {
                    eprintln!("❌ AOT compilation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Check { project } => {
            if !plugin.can_handle(&project) {
                println!("❌ Not a valid Rust project");
                std::process::exit(1);
            }

            let missing_deps = plugin.check_dependencies();
            if missing_deps.is_empty() {
                println!("✅ All dependencies are available");
            } else {
                println!("❌ Missing dependencies:");
                for dep in missing_deps {
                    println!("  - {}", dep);
                }
                std::process::exit(1);
            }
        }

        Commands::Info => {
            println!("WasmRust v{}", env!("CARGO_PKG_VERSION"));
            println!("Rust WebAssembly compiler");
        }
    }
}
