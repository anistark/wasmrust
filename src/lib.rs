use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Deserialize)]
struct CargoTomlFull {
    package: PackageFull,
}

#[derive(Deserialize)]
struct PackageFull {
    name: String,
    version: String,
}

#[derive(Error, Debug)]
pub enum WasmRustError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Invalid project: {0}")]
    InvalidProject(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, WasmRustError>;

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    if !from.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(to)?;

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&from_path, &to_path)?;
        } else {
            std::fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileConfig {
    pub project_path: String,
    pub output_dir: String,
    pub optimization: OptimizationLevel,
    pub target_type: TargetType,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    Wasm,
    WebApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub wasm_path: String,
    pub js_path: Option<String>,
    pub additional_files: Vec<String>,
    pub is_webapp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub project_type: ProjectType,
    pub build_strategy: BuildStrategy,
    pub frameworks: Vec<String>,
    pub dependencies: DependencyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    StandardWasm,
    WasmBindgen,
    WebApplication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStrategy {
    Cargo,
    WasmPack,
    Trunk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub required: Vec<DependencyCheck>,
    pub optional: Vec<DependencyCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCheck {
    pub name: String,
    pub available: bool,
    pub reason: String,
}

#[derive(Clone)]
pub struct WasmRustPlugin;

impl WasmRustPlugin {
    pub fn new() -> Self {
        Self
    }

    pub fn can_handle(&self, project_path: &str) -> bool {
        let cargo_toml = Path::new(project_path).join("Cargo.toml");
        cargo_toml.exists()
    }

    pub fn check_dependencies(&self) -> Vec<String> {
        let mut missing = Vec::new();

        if !self.is_tool_available("cargo") {
            missing.push("cargo (Rust toolchain)".to_string());
        }

        if !self.is_tool_available("rustc") {
            missing.push("rustc (Rust compiler)".to_string());
        }

        if !self.is_wasm_target_installed() {
            missing.push("wasm32-unknown-unknown target".to_string());
        }

        missing
    }

    pub fn analyze_project(&self, project_path: &str) -> Result<ProjectInfo> {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(WasmRustError::InvalidProject(
                "No Cargo.toml found".to_string(),
            ));
        }

        let content = fs::read_to_string(&cargo_toml_path)?;

        let cargo_toml: CargoTomlFull =
            toml::from_str(&content).map_err(WasmRustError::TomlParse)?;

        let name = cargo_toml.package.name.clone();
        let version = cargo_toml.package.version.clone();

        let (project_type, frameworks) =
            self.detect_project_type_and_frameworks(project_path, &content);

        let build_strategy = self.determine_build_strategy(project_path, &project_type);

        let dependencies = self.check_dependencies_comprehensive(&project_type, &build_strategy);

        Ok(ProjectInfo {
            name,
            version,
            project_type,
            build_strategy,
            frameworks,
            dependencies,
        })
    }

    fn detect_project_type_and_frameworks(
        &self,
        project_path: &str,
        cargo_toml_content: &str,
    ) -> (ProjectType, Vec<String>) {
        let mut frameworks = Vec::new();

        let web_frameworks = [
            "yew", "leptos", "dioxus", "sycamore", "mogwai", "seed", "percy", "iced", "dodrio",
            "smithy",
        ];

        for framework in web_frameworks {
            if cargo_toml_content.contains(framework) {
                frameworks.push(framework.to_string());
            }
        }

        let wasm_bindgen_deps = ["wasm-bindgen", "web-sys", "js-sys"];
        let has_wasm_bindgen = wasm_bindgen_deps
            .iter()
            .any(|dep| cargo_toml_content.contains(dep));

        if cargo_toml_content.contains("trunk") {
            frameworks.push("trunk".to_string());
        }

        let project_type = if !frameworks.is_empty() || self.is_rust_web_application(project_path) {
            ProjectType::WebApplication
        } else if has_wasm_bindgen {
            ProjectType::WasmBindgen
        } else {
            ProjectType::StandardWasm
        };

        (project_type, frameworks)
    }

    fn determine_build_strategy(
        &self,
        project_path: &str,
        project_type: &ProjectType,
    ) -> BuildStrategy {
        match project_type {
            ProjectType::StandardWasm => BuildStrategy::Cargo,
            ProjectType::WasmBindgen => BuildStrategy::WasmPack,
            ProjectType::WebApplication => {
                let uses_trunk = Path::new(project_path).join("Trunk.toml").exists()
                    || Path::new(project_path).join("trunk.toml").exists();

                if uses_trunk {
                    BuildStrategy::Trunk
                } else {
                    BuildStrategy::WasmPack
                }
            }
        }
    }

    fn check_dependencies_comprehensive(
        &self,
        _project_type: &ProjectType,
        build_strategy: &BuildStrategy,
    ) -> DependencyStatus {
        let mut required = Vec::new();
        let mut optional = Vec::new();

        required.push(DependencyCheck {
            name: "cargo".to_string(),
            available: self.is_tool_available("cargo"),
            reason: "Rust build tool".to_string(),
        });

        required.push(DependencyCheck {
            name: "rustc".to_string(),
            available: self.is_tool_available("rustc"),
            reason: "Rust compiler".to_string(),
        });

        required.push(DependencyCheck {
            name: "wasm32-unknown-unknown".to_string(),
            available: self.is_wasm_target_installed(),
            reason: "WebAssembly compilation target".to_string(),
        });

        match build_strategy {
            BuildStrategy::WasmPack => {
                required.push(DependencyCheck {
                    name: "wasm-pack".to_string(),
                    available: self.is_tool_available("wasm-pack"),
                    reason: "Required for wasm-bindgen projects".to_string(),
                });
            }
            BuildStrategy::Trunk => {
                required.push(DependencyCheck {
                    name: "trunk".to_string(),
                    available: self.is_tool_available("trunk"),
                    reason: "Required for web application builds".to_string(),
                });

                optional.push(DependencyCheck {
                    name: "wasm-pack".to_string(),
                    available: self.is_tool_available("wasm-pack"),
                    reason: "Useful for optimized builds".to_string(),
                });
            }
            BuildStrategy::Cargo => {
                optional.push(DependencyCheck {
                    name: "wasm-pack".to_string(),
                    available: self.is_tool_available("wasm-pack"),
                    reason: "Useful for advanced WASM features".to_string(),
                });
            }
        }

        optional.push(DependencyCheck {
            name: "rustup".to_string(),
            available: self.is_tool_available("rustup"),
            reason: "Rust toolchain manager".to_string(),
        });

        optional.push(DependencyCheck {
            name: "wasm-opt".to_string(),
            available: self.is_tool_available("wasm-opt"),
            reason: "WebAssembly optimizer".to_string(),
        });

        DependencyStatus { required, optional }
    }

    pub fn compile(&self, config: &CompileConfig) -> Result<CompileResult> {
        if let Some(parent) = Path::new(&config.output_dir).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&config.output_dir)?;

        if self.uses_wasm_bindgen(&config.project_path) {
            if self.is_rust_web_application(&config.project_path) {
                self.compile_web_application(config)
            } else {
                self.compile_wasm_bindgen(config)
            }
        } else {
            self.compile_standard_wasm(config)
        }
    }

    fn uses_wasm_bindgen(&self, project_path: &str) -> bool {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");

        if let Ok(cargo_toml) = fs::read_to_string(cargo_toml_path) {
            cargo_toml.contains("wasm-bindgen")
                || cargo_toml.contains("web-sys")
                || cargo_toml.contains("js-sys")
        } else {
            false
        }
    }

    fn is_rust_web_application(&self, project_path: &str) -> bool {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");

        if let Ok(cargo_toml) = fs::read_to_string(cargo_toml_path) {
            if !self.uses_wasm_bindgen(project_path) {
                return false;
            }

            let web_frameworks = [
                "yew", "leptos", "dioxus", "sycamore", "mogwai", "seed", "percy", "iced", "dodrio",
                "smithy", "trunk",
            ];

            for framework in web_frameworks {
                if cargo_toml.contains(framework) {
                    return true;
                }
            }

            if cargo_toml.contains("[lib]") && cargo_toml.contains("cdylib") {
                if Path::new(project_path).join("index.html").exists() {
                    return true;
                }

                let potential_static_dirs = ["public", "static", "assets", "dist", "www"];
                for dir in potential_static_dirs {
                    if Path::new(project_path).join(dir).exists() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn compile_standard_wasm(&self, config: &CompileConfig) -> Result<CompileResult> {
        self.ensure_wasm32_target(config.verbose)?;

        let mut args = vec!["build", "--target", "wasm32-unknown-unknown"];

        match config.optimization {
            OptimizationLevel::Debug => {}
            OptimizationLevel::Release => args.push("--release"),
            OptimizationLevel::Size => {
                args.push("--release");
            }
        }

        if config.verbose {
            println!("Running: cargo {}", args.join(" "));
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(WasmRustError::CompilationFailed(format!(
                "stdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let profile = match config.optimization {
            OptimizationLevel::Debug => "debug",
            _ => "release",
        };

        let wasm_name = self.get_package_name(&config.project_path)?;
        let wasm_path = Path::new(&config.project_path)
            .join("target/wasm32-unknown-unknown")
            .join(profile)
            .join(format!("{}.wasm", wasm_name));

        if !wasm_path.exists() {
            return Err(WasmRustError::CompilationFailed(format!(
                "WASM file not found at: {}",
                wasm_path.display()
            )));
        }

        let output_wasm = Path::new(&config.output_dir).join(format!("{}.wasm", wasm_name));
        fs::copy(&wasm_path, &output_wasm)?;

        Ok(CompileResult {
            wasm_path: output_wasm.to_string_lossy().to_string(),
            js_path: None,
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn compile_wasm_bindgen(&self, config: &CompileConfig) -> Result<CompileResult> {
        if !self.is_tool_available("wasm-pack") {
            return Err(WasmRustError::ToolNotFound(
                "wasm-pack is required for wasm-bindgen projects".to_string(),
            ));
        }

        let mut args = vec!["build", "--target", "web"];

        match config.optimization {
            OptimizationLevel::Debug => args.push("--dev"),
            OptimizationLevel::Release => args.push("--release"),
            OptimizationLevel::Size => {
                args.push("--release");
            }
        }

        args.extend(["--out-dir", &config.output_dir]);

        if config.verbose {
            println!("Running: wasm-pack {}", args.join(" "));
        }

        let output = Command::new("wasm-pack")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(WasmRustError::CompilationFailed(format!(
                "stdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let package_name = self.get_package_name(&config.project_path)?;
        let wasm_path = Path::new(&config.output_dir).join(format!("{}_bg.wasm", package_name));
        let js_path = Path::new(&config.output_dir).join(format!("{}.js", package_name));

        Ok(CompileResult {
            wasm_path: wasm_path.to_string_lossy().to_string(),
            js_path: Some(js_path.to_string_lossy().to_string()),
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn compile_web_application(&self, config: &CompileConfig) -> Result<CompileResult> {
        let uses_trunk = Path::new(&config.project_path).join("Trunk.toml").exists()
            || Path::new(&config.project_path).join("trunk.toml").exists();

        if uses_trunk && self.is_tool_available("trunk") {
            self.compile_with_trunk(config)
        } else {
            self.compile_wasm_bindgen(config)
        }
    }

    fn compile_with_trunk(&self, config: &CompileConfig) -> Result<CompileResult> {
        let mut args = vec!["build"];

        match config.optimization {
            OptimizationLevel::Debug => {}
            OptimizationLevel::Release => args.push("--release"),
            OptimizationLevel::Size => {
                args.extend(["--release", "--minify"]);
            }
        }

        args.extend(["--dist", "dist"]);

        if config.verbose {
            println!(
                "Running: trunk {} (from directory: {})",
                args.join(" "),
                config.project_path
            );
            println!("Expected output directory: {}/dist", config.project_path);
        }

        let output = Command::new("trunk")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if config.verbose {
            println!("Trunk stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Trunk stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(WasmRustError::CompilationFailed(format!(
                "stdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        let project_dist = Path::new(&config.project_path).join("dist");
        let index_in_project_dist = project_dist.join("index.html");

        if config.verbose {
            println!(
                "Checking for index.html at: {}",
                index_in_project_dist.display()
            );
            if project_dist.exists() {
                println!("Contents of project dist directory:");
                if let Ok(entries) = std::fs::read_dir(&project_dist) {
                    for entry in entries.flatten() {
                        println!("  - {}", entry.file_name().to_string_lossy());
                    }
                }
            } else {
                println!("Project dist directory doesn't exist");
            }
        }

        if index_in_project_dist.exists() {
            if project_dist != Path::new(&config.output_dir) {
                fs::create_dir_all(&config.output_dir)?;
                copy_dir_recursive(&project_dist, Path::new(&config.output_dir))?;
            }

            let final_index = Path::new(&config.output_dir).join("index.html");
            return Ok(CompileResult {
                wasm_path: config.output_dir.clone(),
                js_path: Some(final_index.to_string_lossy().to_string()),
                additional_files: Vec::new(),
                is_webapp: true,
            });
        }

        let index_path = Path::new(&config.output_dir).join("index.html");
        if !index_path.exists() {
            return Err(WasmRustError::CompilationFailed(format!(
                "No index.html generated by trunk. Checked: {} and {}",
                index_in_project_dist.display(),
                index_path.display()
            )));
        }

        Ok(CompileResult {
            wasm_path: config.output_dir.clone(),
            js_path: Some(index_path.to_string_lossy().to_string()),
            additional_files: Vec::new(),
            is_webapp: true,
        })
    }

    fn get_package_name(&self, project_path: &str) -> Result<String> {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        let content = fs::read_to_string(cargo_toml_path)?;

        let cargo_toml: CargoTomlFull = toml::from_str(&content)?;
        Ok(cargo_toml.package.name.replace("-", "_"))
    }

    fn ensure_wasm32_target(&self, verbose: bool) -> Result<()> {
        if !self.is_wasm_target_installed() {
            if verbose {
                println!("Installing wasm32-unknown-unknown target...");
            }

            let output = Command::new("rustup")
                .args(["target", "add", "wasm32-unknown-unknown"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(WasmRustError::CompilationFailed(format!(
                    "Failed to install wasm32 target: {}",
                    stderr
                )));
            }
        }
        Ok(())
    }

    fn is_wasm_target_installed(&self) -> bool {
        Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("wasm32-unknown-unknown")
            })
            .unwrap_or(false)
    }

    fn is_tool_available(&self, tool: &str) -> bool {
        if let Ok(output) = Command::new(tool).arg("--version").output() {
            return output.status.success();
        }

        let which_cmd = if cfg!(target_os = "windows") {
            "where"
        } else {
            "which"
        };

        Command::new(which_cmd)
            .arg(tool)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn run_for_execution(&self, project_path: &str, output_dir: &str) -> Result<String> {
        let config = CompileConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization: OptimizationLevel::Release,
            target_type: TargetType::Wasm,
            verbose: false,
        };

        let result = self.compile(&config)?;
        Ok(result.js_path.unwrap_or(result.wasm_path))
    }

    pub fn run_for_execution_with_config(
        &self,
        project_path: &str,
        output_dir: &str,
        optimization: OptimizationLevel,
    ) -> Result<String> {
        let config = CompileConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization,
            target_type: TargetType::Wasm,
            verbose: false,
        };

        let result = self.compile(&config)?;
        Ok(result.js_path.unwrap_or(result.wasm_path))
    }

    pub fn verify_dependencies(&self) -> Result<()> {
        let missing = self.check_dependencies();
        if !missing.is_empty() {
            return Err(WasmRustError::ToolNotFound(format!(
                "Missing dependencies: {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }

    pub fn get_project_info(&self, project_path: &str) -> Result<ProjectInfo> {
        self.analyze_project(project_path)
    }

    pub fn compile_for_execution(
        &self,
        project_path: &str,
        output_dir: &str,
        optimization: OptimizationLevel,
    ) -> Result<String> {
        let config = CompileConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization,
            target_type: TargetType::Wasm,
            verbose: false,
        };

        let result = self.compile(&config)?;
        Ok(result.js_path.unwrap_or(result.wasm_path))
    }

    pub fn supports_web_app(&self, project_path: &str) -> bool {
        self.is_rust_web_application(project_path)
    }

    pub fn get_extensions(&self) -> Vec<String> {
        vec!["rs".to_string()]
    }

    pub fn get_entry_files(&self) -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "main.rs".to_string(),
            "lib.rs".to_string(),
        ]
    }
}

impl Default for WasmRustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// WASMRUN INTEGRATION - WasmBuilder Implementation
// ============================================================================

// These types are compatible with wasmrun's builder.rs
#[derive(Debug, Clone)]
pub struct WasmrunBuildConfig {
    pub project_path: String,
    pub output_dir: String,
    pub optimization_level: WasmrunOptimizationLevel,
    pub verbose: bool,
    pub watch: bool,
}

#[derive(Debug, Clone)]
pub enum WasmrunOptimizationLevel {
    Debug,
    Release,
    Size,
}

#[derive(Debug, Clone)]
pub struct WasmrunBuildResult {
    pub wasm_path: String,
    pub js_path: Option<String>,
    pub additional_files: Vec<String>,
    pub is_wasm_bindgen: bool,
}

// Error type compatible with wasmrun
#[derive(Debug)]
pub enum WasmrunCompilationError {
    BuildFailed { language: String, reason: String },
}

impl std::fmt::Display for WasmrunCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmrunCompilationError::BuildFailed { language, reason } => {
                write!(f, "{} build failed: {}", language, reason)
            }
        }
    }
}

impl std::error::Error for WasmrunCompilationError {}

pub type WasmrunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type WasmrunCompilationResult<T> = std::result::Result<T, WasmrunCompilationError>;

// Trait definition compatible with wasmrun
pub trait WasmrunWasmBuilder: Send + Sync {
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn build(&self, config: &WasmrunBuildConfig) -> WasmrunCompilationResult<WasmrunBuildResult>;
    fn clean(&self, project_path: &str) -> WasmrunResult<()>;
    fn clone_box(&self) -> Box<dyn WasmrunWasmBuilder>;
}

// WasmBuilder implementation for wasmrun
#[derive(Clone)]
pub struct WasmRustWasmBuilder {
    plugin: WasmRustPlugin,
}

impl WasmRustWasmBuilder {
    pub fn new() -> Self {
        Self {
            plugin: WasmRustPlugin::new(),
        }
    }
}

impl WasmrunWasmBuilder for WasmRustWasmBuilder {
    fn can_handle_project(&self, project_path: &str) -> bool {
        self.plugin.can_handle(project_path)
    }

    fn build(&self, config: &WasmrunBuildConfig) -> WasmrunCompilationResult<WasmrunBuildResult> {
        // Convert wasmrun BuildConfig to wasmrust CompileConfig
        let optimization = match config.optimization_level {
            WasmrunOptimizationLevel::Debug => OptimizationLevel::Debug,
            WasmrunOptimizationLevel::Release => OptimizationLevel::Release,
            WasmrunOptimizationLevel::Size => OptimizationLevel::Size,
        };

        let compile_config = CompileConfig {
            project_path: config.project_path.clone(),
            output_dir: config.output_dir.clone(),
            optimization,
            target_type: TargetType::Wasm,
            verbose: config.verbose,
        };

        match self.plugin.compile(&compile_config) {
            Ok(result) => {
                // Calculate is_wasm_bindgen BEFORE moving result.js_path
                let is_wasm_bindgen = result.js_path.is_some();

                // Convert wasmrust CompileResult to wasmrun BuildResult
                Ok(WasmrunBuildResult {
                    wasm_path: result.wasm_path,
                    js_path: result.js_path,
                    additional_files: result.additional_files,
                    is_wasm_bindgen,
                })
            }
            Err(e) => Err(WasmrunCompilationError::BuildFailed {
                language: "rust".to_string(),
                reason: format!("{}", e),
            }),
        }
    }

    fn clean(&self, project_path: &str) -> WasmrunResult<()> {
        // Clean Rust project artifacts
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err("No Cargo.toml found".into());
        }

        let output = Command::new("cargo")
            .args(["clean"])
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to execute cargo clean: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("cargo clean failed: {}", stderr).into());
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn WasmrunWasmBuilder> {
        Box::new(self.clone())
    }
}

impl Default for WasmRustWasmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DYNAMIC LOADING C INTERFACE
// ============================================================================

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct BuildConfigC {
    pub project_path: *const c_char,
    pub output_dir: *const c_char,
    pub optimization_level: u8, // 0=Debug, 1=Release, 2=Size
    pub verbose: bool,
    pub watch: bool,
}

#[repr(C)]
pub struct BuildResultC {
    pub wasm_path: *mut c_char,
    pub js_path: *mut c_char, // null if None
    pub is_wasm_bindgen: bool,
    pub success: bool,
    pub error_message: *mut c_char, // null if success
}

// Main factory function for creating the builder
#[no_mangle]
pub extern "C" fn create_wasm_builder() -> *mut c_void {
    let builder = Box::new(WasmRustWasmBuilder::new());
    Box::into_raw(builder) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_can_handle_project(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.can_handle_project(path_str)
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_build(
    builder_ptr: *const c_void,
    config: *const BuildConfigC,
) -> *mut BuildResultC {
    if builder_ptr.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let config_c = &*config;

    let project_path = match CStr::from_ptr(config_c.project_path).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let output_dir = match CStr::from_ptr(config_c.output_dir).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let optimization = match config_c.optimization_level {
        0 => WasmrunOptimizationLevel::Debug,
        1 => WasmrunOptimizationLevel::Release,
        2 => WasmrunOptimizationLevel::Size,
        _ => WasmrunOptimizationLevel::Release,
    };

    let build_config = WasmrunBuildConfig {
        project_path,
        output_dir,
        optimization_level: optimization,
        verbose: config_c.verbose,
        watch: config_c.watch,
    };

    match builder.build(&build_config) {
        Ok(result) => {
            let wasm_path = CString::new(result.wasm_path).unwrap();
            let js_path = result.js_path.map(|p| CString::new(p).unwrap());

            let result_c = Box::new(BuildResultC {
                wasm_path: wasm_path.into_raw(),
                js_path: js_path.map(|p| p.into_raw()).unwrap_or(ptr::null_mut()),
                is_wasm_bindgen: result.is_wasm_bindgen,
                success: true,
                error_message: ptr::null_mut(),
            });

            Box::into_raw(result_c)
        }
        Err(e) => {
            let error_msg = CString::new(format!("{}", e)).unwrap();
            let result_c = Box::new(BuildResultC {
                wasm_path: ptr::null_mut(),
                js_path: ptr::null_mut(),
                is_wasm_bindgen: false,
                success: false,
                error_message: error_msg.into_raw(),
            });

            Box::into_raw(result_c)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_clean(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.clean(path_str).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_clone_box(builder_ptr: *const c_void) -> *mut c_void {
    if builder_ptr.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let cloned = builder.clone_box();
    Box::into_raw(cloned) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_drop(builder_ptr: *mut c_void) {
    if !builder_ptr.is_null() {
        let _ = Box::from_raw(builder_ptr as *mut WasmRustWasmBuilder);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_free_build_result(result_ptr: *mut BuildResultC) {
    if result_ptr.is_null() {
        return;
    }

    let result = Box::from_raw(result_ptr);

    if !result.wasm_path.is_null() {
        let _ = CString::from_raw(result.wasm_path);
    }

    if !result.js_path.is_null() {
        let _ = CString::from_raw(result.js_path);
    }

    if !result.error_message.is_null() {
        let _ = CString::from_raw(result.error_message);
    }
}

// Additional utility functions for wasmrun integration
#[no_mangle]
pub unsafe extern "C" fn wasmrust_get_extensions(
    builder_ptr: *const c_void,
    extensions_out: *mut *mut *mut c_char,
    count_out: *mut usize,
) -> bool {
    if builder_ptr.is_null() || extensions_out.is_null() || count_out.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let extensions = builder.plugin.get_extensions();

    let mut c_extensions = Vec::new();
    for ext in extensions {
        if let Ok(c_ext) = CString::new(ext) {
            c_extensions.push(c_ext.into_raw());
        }
    }

    let len = c_extensions.len();
    let ptr = c_extensions.into_boxed_slice();

    *extensions_out = Box::into_raw(ptr) as *mut *mut c_char;
    *count_out = len;

    true
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_get_entry_files(
    builder_ptr: *const c_void,
    entry_files_out: *mut *mut *mut c_char,
    count_out: *mut usize,
) -> bool {
    if builder_ptr.is_null() || entry_files_out.is_null() || count_out.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let entry_files = builder.plugin.get_entry_files();

    let mut c_entry_files = Vec::new();
    for file in entry_files {
        if let Ok(c_file) = CString::new(file) {
            c_entry_files.push(c_file.into_raw());
        }
    }

    let len = c_entry_files.len();
    let ptr = c_entry_files.into_boxed_slice();

    *entry_files_out = Box::into_raw(ptr) as *mut *mut c_char;
    *count_out = len;

    true
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_supports_web_app(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.plugin.supports_web_app(path_str)
}

#[no_mangle]
pub unsafe extern "C" fn wasmrust_verify_dependencies(builder_ptr: *const c_void) -> bool {
    if builder_ptr.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmRustWasmBuilder);
    builder.plugin.verify_dependencies().is_ok()
}

// Free functions for cleaning up string arrays
#[no_mangle]
pub unsafe extern "C" fn wasmrust_free_string_array(array_ptr: *mut *mut c_char, count: usize) {
    if array_ptr.is_null() {
        return;
    }

    let array = Box::from_raw(std::slice::from_raw_parts_mut(array_ptr, count));
    for ptr in array.iter() {
        if !ptr.is_null() {
            let _ = CString::from_raw(*ptr);
        }
    }
}

// Plugin metadata for wasmrun discovery
#[no_mangle]
pub static WASMRUST_PLUGIN_NAME: &[u8] = b"wasmrust\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_VERSION: &[u8] = b"0.1.5\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_DESCRIPTION: &[u8] = b"Rust WebAssembly compiler plugin\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_AUTHOR: &[u8] = b"Kumar Anirudha\0";

// Plugin capabilities flags
#[no_mangle]
pub static WASMRUST_SUPPORTS_WASM: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_WEBAPP: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_LIVE_RELOAD: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_OPTIMIZATION: bool = true;
