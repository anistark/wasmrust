use clap::{Parser, Subcommand};
use wasmrust::{CompileConfig, OptimizationLevel, TargetType, WasmRustPlugin};

#[derive(Parser)]
#[command(name = "wasmrust")]
#[command(about = "Rust WebAssembly plugin for Wasmrun")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Rust WebAssembly project (default command)
    #[command(alias = "r")]
    Run {
        #[arg(short, long, default_value = ".")]
        project: String,
        
        #[arg(short, long, default_value = "./dist")]
        output: String,
        
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,
        
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Compile a Rust project to WebAssembly
    #[command(alias = "c")]
    Compile {
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
                println!("🚀 Running Rust WebAssembly project...");
            }

            match plugin.run_for_execution_with_config(&project, &output, optimization.into()) {
                Ok(entry_point) => {
                    if verbose {
                        println!("✅ Project ready for execution!");
                        println!("📦 Entry point: {}", entry_point);
                    } else {
                        // For scripting
                        println!("{}", entry_point);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to prepare project for execution: {}", e);
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
            let config = CompileConfig {
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

            match plugin.compile(&config) {
                Ok(result) => {
                    println!("✅ Compilation completed successfully!");
                    println!("📦 WASM: {}", result.wasm_path);
                    
                    if let Some(js_path) = result.js_path {
                        println!("📝 JS: {}", js_path);
                    }
                    
                    if result.is_webapp {
                        println!("🌐 Web application compiled successfully");
                    }

                    if !result.additional_files.is_empty() {
                        println!("📄 Additional files: {}", result.additional_files.len());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Compilation failed: {}", e);
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
            println!();
            println!("Commands:");
            println!("  run      Run a Rust WebAssembly project (default)");
            println!("  compile  Compile a Rust project to WebAssembly");
            println!("  check    Check project dependencies");
            println!("  info     Show version and usage information");
        }
    }
}
