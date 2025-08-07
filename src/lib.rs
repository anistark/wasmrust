use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

// Core plugin types - defined locally since wasmrun-core doesn't exist
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Builtin,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub compile_wasm: bool,
    pub compile_webapp: bool,
    pub live_reload: bool,
    pub optimization: bool,
    pub custom_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSource {
    CratesIo { name: String, version: String },
    Git { url: String, rev: Option<String> },
    Local { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub extensions: Vec<String>,
    pub entry_files: Vec<String>,
    pub plugin_type: PluginType,
    pub source: Option<PluginSource>,
    pub dependencies: Vec<String>,
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub input: String,
    pub output_dir: String,
    pub optimization: OptimizationLevel,
    pub target_type: String,
    pub verbose: bool,
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub output_path: String,
    pub language: String,
    pub optimization_level: OptimizationLevel,
    pub build_time: std::time::Duration,
    pub file_size: u64,
}

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Build failed for {language}: {reason}")]
    BuildFailed { language: String, reason: String },

    #[error("Tool execution failed - {tool}: {reason}")]
    ToolExecutionFailed { tool: String, reason: String },

    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },
}

pub type CompilationResult<T> = std::result::Result<T, CompilationError>;

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

pub type WasmRustResult<T> = std::result::Result<T, WasmRustError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileConfig {
    pub project_path: String,
    pub output_dir: String,
    pub optimization: OptimizationLevel,
    pub target_type: TargetType,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

// Plugin trait definitions
pub trait Plugin {
    fn info(&self) -> &PluginInfo;
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn get_builder(&self) -> Box<dyn WasmBuilder>;
}

pub trait WasmBuilder {
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn build(&self, config: &BuildConfig) -> CompilationResult<BuildResult>;
    fn check_dependencies(&self) -> Vec<String>;
    fn validate_project(&self, project_path: &str) -> CompilationResult<()>;
    fn clean(&self, project_path: &str) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn clone_box(&self) -> Box<dyn WasmBuilder>;
    fn language_name(&self) -> &str;
    fn entry_file_candidates(&self) -> &[&str];
    fn supported_extensions(&self) -> &[&str];
}

fn copy_dir_recursive(from: &Path, to: &Path) -> WasmRustResult<()> {
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

    pub fn inspect_project(&self, project_path: &str) -> WasmRustResult<ProjectInfo> {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(WasmRustError::InvalidProject(
                "No Cargo.toml found".to_string(),
            ));
        }

        let content = fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml: CargoTomlFull = toml::from_str(&content)?;

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

    pub fn compile(&self, config: &CompileConfig) -> WasmRustResult<CompileResult> {
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

    pub fn compile_for_aot(&self, project_path: &str, output_dir: &str) -> WasmRustResult<String> {
        let config = CompileConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization: OptimizationLevel::Release,
            target_type: if self.is_rust_web_application(project_path) {
                TargetType::WebApp
            } else {
                TargetType::Wasm
            },
            verbose: false,
        };

        let result = self.compile(&config)?;
        self.get_primary_output_file(&result)
    }

    pub fn compile_for_aot_with_optimization(
        &self,
        project_path: &str,
        output_dir: &str,
        optimization: OptimizationLevel,
    ) -> WasmRustResult<String> {
        let config = CompileConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization,
            target_type: if self.is_rust_web_application(project_path) {
                TargetType::WebApp
            } else {
                TargetType::Wasm
            },
            verbose: false,
        };

        let result = self.compile(&config)?;
        self.get_primary_output_file(&result)
    }

    fn get_primary_output_file(&self, result: &CompileResult) -> WasmRustResult<String> {
        if result.is_webapp {
            Ok(result.wasm_path.clone())
        } else if let Some(js_path) = &result.js_path {
            Ok(js_path.clone())
        } else {
            Ok(result.wasm_path.clone())
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

    fn compile_standard_wasm(&self, config: &CompileConfig) -> WasmRustResult<CompileResult> {
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
                "stdout: {stdout}\nstderr: {stderr}",
            )));
        }

        let profile = match config.optimization {
            OptimizationLevel::Debug => "debug",
            _ => "release",
        };

        let wasm_name = self.get_package_name(&config.project_path)?;
        let target_dir = Path::new(&config.project_path)
            .join("target/wasm32-unknown-unknown")
            .join(profile);

        // Try multiple potential WASM file names
        let potential_names = vec![
            format!("{wasm_name}.wasm"),
            format!("{}.wasm", wasm_name.replace("_", "-")),
            format!("{}.wasm", wasm_name.replace("-", "_")),
        ];

        let mut wasm_path = None;
        for name in &potential_names {
            let path = target_dir.join(name);
            if path.exists() {
                wasm_path = Some(path);
                break;
            }
        }

        let wasm_path = wasm_path.ok_or_else(|| {
            let mut error_msg = "WASM file not found. Tried:\n".to_string();
            for name in &potential_names {
                let path = target_dir.join(name);
                error_msg.push_str(&format!("  - {}\n", path.display()));
            }

            // List actual files in the directory for debugging
            if let Ok(entries) = std::fs::read_dir(&target_dir) {
                error_msg.push_str("Files found in target directory:\n");
                for entry in entries.flatten() {
                    error_msg.push_str(&format!("  - {}\n", entry.file_name().to_string_lossy()));
                }
            } else {
                error_msg.push_str(&format!(
                    "Target directory doesn't exist: {}\n",
                    target_dir.display()
                ));
            }

            WasmRustError::CompilationFailed(error_msg)
        })?;

        let output_wasm = Path::new(&config.output_dir).join(format!("{wasm_name}.wasm"));
        fs::copy(&wasm_path, &output_wasm)?;

        Ok(CompileResult {
            wasm_path: output_wasm.to_string_lossy().to_string(),
            js_path: None,
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn compile_wasm_bindgen(&self, config: &CompileConfig) -> WasmRustResult<CompileResult> {
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
                "stdout: {stdout}\nstderr: {stderr}"
            )));
        }

        let package_name = self.get_package_name(&config.project_path)?;
        let wasm_path = Path::new(&config.output_dir).join(format!("{package_name}_bg.wasm"));
        let js_path = Path::new(&config.output_dir).join(format!("{package_name}.js"));

        Ok(CompileResult {
            wasm_path: wasm_path.to_string_lossy().to_string(),
            js_path: Some(js_path.to_string_lossy().to_string()),
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn compile_web_application(&self, config: &CompileConfig) -> WasmRustResult<CompileResult> {
        let uses_trunk = Path::new(&config.project_path).join("Trunk.toml").exists()
            || Path::new(&config.project_path).join("trunk.toml").exists();

        if uses_trunk && self.is_tool_available("trunk") {
            self.compile_with_trunk(config)
        } else {
            self.compile_wasm_bindgen(config)
        }
    }

    fn compile_with_trunk(&self, config: &CompileConfig) -> WasmRustResult<CompileResult> {
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
                "stdout: {stdout}\nstderr: {stderr}",
            )));
        }

        let project_dist = Path::new(&config.project_path).join("dist");
        let index_in_project_dist = project_dist.join("index.html");

        if config.verbose {
            println!(
                "Checking for index.html at: {}",
                index_in_project_dist.display()
            );
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

    fn get_package_name(&self, project_path: &str) -> WasmRustResult<String> {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        let content = fs::read_to_string(cargo_toml_path)?;
        let cargo_toml: CargoTomlFull = toml::from_str(&content)?;
        Ok(cargo_toml.package.name.replace("-", "_"))
    }

    fn ensure_wasm32_target(&self, verbose: bool) -> WasmRustResult<()> {
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
                    "Failed to install wasm32 target: {stderr}",
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

    pub fn is_tool_available(&self, tool: &str) -> bool {
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

    pub fn get_watch_paths(&self, project_path: &str) -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        let project_path = std::path::Path::new(project_path);

        paths.push(project_path.join("Cargo.toml"));

        if project_path.join("src").exists() {
            paths.push(project_path.join("src"));
        }

        if project_path.join("Trunk.toml").exists() {
            paths.push(project_path.join("Trunk.toml"));
        }

        for asset_dir in &["assets", "static", "public"] {
            if project_path.join(asset_dir).exists() {
                paths.push(project_path.join(asset_dir));
            }
        }

        paths
    }

    pub fn verify_dependencies(&self) -> WasmRustResult<()> {
        let missing = self.check_dependencies();
        if !missing.is_empty() {
            return Err(WasmRustError::ToolNotFound(format!(
                "Missing dependencies: {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }

    pub fn get_project_info(&self, project_path: &str) -> WasmRustResult<ProjectInfo> {
        self.inspect_project(project_path)
    }

    pub fn supports_web_app(&self, project_path: &str) -> bool {
        self.is_rust_web_application(project_path)
    }

    pub fn get_extensions(&self) -> Vec<String> {
        vec!["rs".to_string(), "toml".to_string()]
    }

    pub fn get_entry_files(&self) -> Vec<String> {
        vec![
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ]
    }
}

impl Default for WasmRustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// Plugin trait implementations
pub struct WasmrustPlugin {
    inner: WasmRustPlugin,
    info: PluginInfo,
}

impl WasmrustPlugin {
    pub fn new() -> Self {
        let info = PluginInfo {
            name: "wasmrust".to_string(),
            version: "0.3.0".to_string(),
            description: "Rust to WebAssembly compiler with wasm-bindgen support".to_string(),
            author: "Kumar Anirudha".to_string(),
            extensions: vec!["rs".to_string(), "toml".to_string()],
            entry_files: vec![
                "Cargo.toml".to_string(),
                "src/main.rs".to_string(),
                "src/lib.rs".to_string(),
            ],
            plugin_type: PluginType::External,
            source: Some(PluginSource::CratesIo {
                name: "wasmrust".to_string(),
                version: "0.3.0".to_string(),
            }),
            dependencies: vec![
                "cargo".to_string(),
                "rustc".to_string(),
                "wasm-pack".to_string(),
            ],
            capabilities: PluginCapabilities {
                compile_wasm: true,
                compile_webapp: true,
                live_reload: true,
                optimization: true,
                custom_targets: vec!["wasm32-unknown-unknown".to_string(), "web".to_string()],
            },
        };

        Self {
            inner: WasmRustPlugin::new(),
            info,
        }
    }
}

impl Default for WasmrustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WasmrustPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn can_handle_project(&self, project_path: &str) -> bool {
        self.inner.can_handle(project_path)
    }

    fn get_builder(&self) -> Box<dyn WasmBuilder> {
        Box::new(WasmrustBuilder::new())
    }
}

pub struct WasmrustBuilder {
    inner: WasmRustPlugin,
}

impl WasmrustBuilder {
    pub fn new() -> Self {
        Self {
            inner: WasmRustPlugin::new(),
        }
    }
}

impl Default for WasmrustBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmBuilder for WasmrustBuilder {
    fn can_handle_project(&self, project_path: &str) -> bool {
        self.inner.can_handle(project_path)
    }

    fn build(&self, config: &BuildConfig) -> CompilationResult<BuildResult> {
        let start_time = std::time::Instant::now();

        let optimization = config.optimization.clone();
        let target_type = if config.target_type == "webapp"
            || self.inner.is_rust_web_application(&config.input)
        {
            TargetType::WebApp
        } else {
            TargetType::Wasm
        };

        let compile_config = CompileConfig {
            project_path: config.input.clone(),
            output_dir: config.output_dir.clone(),
            optimization: optimization.clone(),
            target_type,
            verbose: config.verbose,
        };

        match self.inner.compile(&compile_config) {
            Ok(result) => {
                let build_time = start_time.elapsed();

                let file_size = if result.is_webapp {
                    std::fs::read_dir(&result.wasm_path)
                        .ok()
                        .and_then(|entries| {
                            entries
                                .filter_map(|entry| entry.ok())
                                .find(|entry| {
                                    entry.path().extension().is_some_and(|ext| ext == "wasm")
                                })
                                .and_then(|entry| std::fs::metadata(entry.path()).ok())
                                .map(|metadata| metadata.len())
                        })
                        .unwrap_or(0)
                } else {
                    std::fs::metadata(&result.wasm_path)
                        .map(|m| m.len())
                        .unwrap_or(0)
                };

                Ok(BuildResult {
                    output_path: result.js_path.unwrap_or(result.wasm_path),
                    language: "rust".to_string(),
                    optimization_level: optimization,
                    build_time,
                    file_size,
                })
            }
            Err(e) => Err(CompilationError::BuildFailed {
                language: "rust".to_string(),
                reason: format!("{e}"),
            }),
        }
    }

    fn check_dependencies(&self) -> Vec<String> {
        self.inner.check_dependencies()
    }

    fn validate_project(&self, project_path: &str) -> CompilationResult<()> {
        if !self.inner.can_handle(project_path) {
            return Err(CompilationError::BuildFailed {
                language: "rust".to_string(),
                reason: format!("Project at '{project_path}' is not a valid Rust project"),
            });
        }
        Ok(())
    }

    fn clean(&self, project_path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .args(["clean"])
            .current_dir(project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Clean failed: {stderr}").into());
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn WasmBuilder> {
        Box::new(WasmrustBuilder::new())
    }

    fn language_name(&self) -> &str {
        "rust"
    }

    fn entry_file_candidates(&self) -> &[&str] {
        &["Cargo.toml", "src/main.rs", "src/lib.rs"]
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs", "toml"]
    }
}

// Plugin factory function for Wasmrun integration
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(WasmrustPlugin::new())
}

// C interface for dynamic loading
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct BuildConfigC {
    pub input: *const c_char,
    pub output_dir: *const c_char,
    pub optimization: u8, // 0=Debug, 1=Release, 2=Size
    pub target_type: *const c_char,
    pub verbose: bool,
    pub watch: bool,
}

#[repr(C)]
pub struct BuildResultC {
    pub output_path: *mut c_char,
    pub language: *mut c_char,
    pub file_size: u64,
    pub success: bool,
    pub error_message: *mut c_char,
}

#[no_mangle]
pub extern "C" fn wasmrun_plugin_create() -> *mut c_void {
    let plugin = Box::new(WasmrustPlugin::new());
    Box::into_raw(plugin) as *mut c_void
}

#[no_mangle]
pub extern "C" fn create_wasm_builder() -> *mut c_void {
    let builder = Box::new(WasmrustBuilder::new());
    Box::into_raw(builder) as *mut c_void
}

/// Checks if the builder can handle the specified project.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - `project_path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn wasmrust_can_handle_project(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.can_handle_project(path_str)
}

/// Builds the project with the given configuration.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - `config` must be a valid pointer to a BuildConfigC
/// - Caller must call `wasmrust_free_build_result` on the returned pointer
#[no_mangle]
pub unsafe extern "C" fn wasmrust_build(
    builder_ptr: *const c_void,
    config: *const BuildConfigC,
) -> *mut BuildResultC {
    if builder_ptr.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    let config_c = &*config;

    let input = match CStr::from_ptr(config_c.input).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let output_dir = match CStr::from_ptr(config_c.output_dir).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let target_type = match CStr::from_ptr(config_c.target_type).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => "wasm".to_string(),
    };

    let optimization = match config_c.optimization {
        0 => OptimizationLevel::Debug,
        1 => OptimizationLevel::Release,
        2 => OptimizationLevel::Size,
        _ => OptimizationLevel::Release,
    };

    let build_config = BuildConfig {
        input,
        output_dir,
        optimization,
        target_type,
        verbose: config_c.verbose,
        watch: config_c.watch,
    };

    match builder.build(&build_config) {
        Ok(result) => {
            let output_path = CString::new(result.output_path).unwrap();
            let language = CString::new(result.language).unwrap();

            let result_c = Box::new(BuildResultC {
                output_path: output_path.into_raw(),
                language: language.into_raw(),
                file_size: result.file_size,
                success: true,
                error_message: ptr::null_mut(),
            });

            Box::into_raw(result_c)
        }
        Err(e) => {
            let error_msg = CString::new(format!("{e}")).unwrap();
            let result_c = Box::new(BuildResultC {
                output_path: ptr::null_mut(),
                language: ptr::null_mut(),
                file_size: 0,
                success: false,
                error_message: error_msg.into_raw(),
            });

            Box::into_raw(result_c)
        }
    }
}

/// Cleans the project build artifacts.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - `project_path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn wasmrust_clean(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.clean(path_str).is_ok()
}

/// Creates a clone of the builder.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - Caller must call `wasmrust_drop` on the returned pointer
#[no_mangle]
pub unsafe extern "C" fn wasmrust_clone_box(builder_ptr: *const c_void) -> *mut c_void {
    if builder_ptr.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    let cloned = builder.clone_box();
    Box::into_raw(cloned) as *mut c_void
}

/// Drops and frees a builder instance.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - The pointer must not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn wasmrust_drop(builder_ptr: *mut c_void) {
    if !builder_ptr.is_null() {
        let _ = Box::from_raw(builder_ptr as *mut WasmrustBuilder);
    }
}

/// Frees a build result and associated memory.
///
/// # Safety
///
/// - `result_ptr` must be a valid pointer to a BuildResultC
/// - The pointer must not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn wasmrust_free_build_result(result_ptr: *mut BuildResultC) {
    if result_ptr.is_null() {
        return;
    }

    let result = Box::from_raw(result_ptr);

    if !result.output_path.is_null() {
        let _ = CString::from_raw(result.output_path);
    }

    if !result.language.is_null() {
        let _ = CString::from_raw(result.language);
    }

    if !result.error_message.is_null() {
        let _ = CString::from_raw(result.error_message);
    }
}

/// Checks if the plugin supports web applications.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
/// - `project_path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn wasmrust_supports_web_app(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    let path_str = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.inner.supports_web_app(path_str)
}

/// Verifies that all required dependencies are available.
///
/// # Safety
///
/// - `builder_ptr` must be a valid pointer to a WasmrustBuilder
#[no_mangle]
pub unsafe extern "C" fn wasmrust_verify_dependencies(builder_ptr: *const c_void) -> bool {
    if builder_ptr.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const WasmrustBuilder);
    builder.inner.verify_dependencies().is_ok()
}

// Plugin metadata for wasmrun discovery
#[no_mangle]
pub static WASMRUST_PLUGIN_NAME: &[u8] = b"wasmrust\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_VERSION: &[u8] = b"0.3.0\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_DESCRIPTION: &[u8] = b"Rust WebAssembly compiler plugin\0";

#[no_mangle]
pub static WASMRUST_PLUGIN_AUTHOR: &[u8] = b"Kumar Anirudha\0";

#[no_mangle]
pub static WASMRUST_SUPPORTS_WASM: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_WEBAPP: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_LIVE_RELOAD: bool = true;

#[no_mangle]
pub static WASMRUST_SUPPORTS_OPTIMIZATION: bool = true;
