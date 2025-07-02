use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmRustError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Invalid project: {0}")]
    InvalidProject(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, WasmRustError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
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
pub struct BuildResult {
    pub wasm_path: String,
    pub js_path: Option<String>,
    pub additional_files: Vec<String>,
    pub is_webapp: bool,
}

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

    pub fn build(&self, config: &BuildConfig) -> Result<BuildResult> {
        if self.uses_wasm_bindgen(&config.project_path) {
            if self.is_rust_web_application(&config.project_path) {
                self.build_web_application(config)
            } else {
                self.build_wasm_bindgen(config)
            }
        } else {
            self.build_standard_wasm(config)
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

    fn build_standard_wasm(&self, config: &BuildConfig) -> Result<BuildResult> {
        self.ensure_wasm32_target()?;

        let mut args = vec!["build", "--target", "wasm32-unknown-unknown"];

        match config.optimization {
            OptimizationLevel::Debug => {}
            OptimizationLevel::Release => args.push("--release"),
            OptimizationLevel::Size => {
                args.push("--release");
            }
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WasmRustError::BuildFailed(stderr.to_string()));
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
            return Err(WasmRustError::BuildFailed(
                "WASM file not found after build".to_string(),
            ));
        }

        let output_wasm = Path::new(&config.output_dir).join(format!("{}.wasm", wasm_name));
        fs::copy(&wasm_path, &output_wasm)?;

        Ok(BuildResult {
            wasm_path: output_wasm.to_string_lossy().to_string(),
            js_path: None,
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn build_wasm_bindgen(&self, config: &BuildConfig) -> Result<BuildResult> {
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

        let output = Command::new("wasm-pack")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WasmRustError::BuildFailed(stderr.to_string()));
        }

        let package_name = self.get_package_name(&config.project_path)?;
        let wasm_path = Path::new(&config.output_dir).join(format!("{}_bg.wasm", package_name));
        let js_path = Path::new(&config.output_dir).join(format!("{}.js", package_name));

        Ok(BuildResult {
            wasm_path: wasm_path.to_string_lossy().to_string(),
            js_path: Some(js_path.to_string_lossy().to_string()),
            additional_files: Vec::new(),
            is_webapp: false,
        })
    }

    fn build_web_application(&self, config: &BuildConfig) -> Result<BuildResult> {
        // Check if project uses Trunk
        let uses_trunk = Path::new(&config.project_path).join("Trunk.toml").exists()
            || Path::new(&config.project_path).join("trunk.toml").exists();

        if uses_trunk && self.is_tool_available("trunk") {
            self.build_with_trunk(config)
        } else {
            // Fall back to wasm-pack for web apps
            self.build_wasm_bindgen(config)
        }
    }

    fn build_with_trunk(&self, config: &BuildConfig) -> Result<BuildResult> {
        let mut args = vec!["build"];

        match config.optimization {
            OptimizationLevel::Debug => {}
            OptimizationLevel::Release => args.push("--release"),
            OptimizationLevel::Size => {
                args.extend(["--release", "--minify"]);
            }
        }

        args.extend(["--dist", &config.output_dir]);

        let output = Command::new("trunk")
            .args(&args)
            .current_dir(&config.project_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WasmRustError::BuildFailed(stderr.to_string()));
        }

        let index_path = Path::new(&config.output_dir).join("index.html");
        if !index_path.exists() {
            return Err(WasmRustError::BuildFailed(
                "No index.html generated by trunk".to_string(),
            ));
        }

        // For web apps, return the dist directory as the "wasm_path"
        // and index.html as the "js_path" (entry point)
        Ok(BuildResult {
            wasm_path: config.output_dir.clone(),
            js_path: Some(index_path.to_string_lossy().to_string()),
            additional_files: Vec::new(),
            is_webapp: true,
        })
    }

    fn get_package_name(&self, project_path: &str) -> Result<String> {
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        let content = fs::read_to_string(cargo_toml_path)?;

        #[derive(Deserialize)]
        struct CargoToml {
            package: Package,
        }

        #[derive(Deserialize)]
        struct Package {
            name: String,
        }

        let cargo_toml: CargoToml = toml::from_str(&content)?;
        Ok(cargo_toml.package.name.replace("-", "_"))
    }

    fn ensure_wasm32_target(&self) -> Result<()> {
        if !self.is_wasm_target_installed() {
            let output = Command::new("rustup")
                .args(["target", "add", "wasm32-unknown-unknown"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(WasmRustError::BuildFailed(format!(
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
        Command::new("which")
            .arg(tool)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Compile for execution (returns JS file for wasm-bindgen, WASM for standard)
    pub fn compile_for_execution(&self, project_path: &str, output_dir: &str) -> Result<String> {
        let config = BuildConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization: OptimizationLevel::Release,
            target_type: TargetType::Wasm,
            verbose: false,
        };

        let result = self.build(&config)?;

        // For wasm-bindgen projects, return JS file for easier loading
        // For standard WASM, return the WASM file
        Ok(result.js_path.unwrap_or(result.wasm_path))
    }

    /// Compile for execution with custom optimization
    pub fn compile_for_execution_with_config(
        &self,
        project_path: &str,
        output_dir: &str,
        optimization: OptimizationLevel,
    ) -> Result<String> {
        let config = BuildConfig {
            project_path: project_path.to_string(),
            output_dir: output_dir.to_string(),
            optimization,
            target_type: TargetType::Wasm,
            verbose: false,
        };

        let result = self.build(&config)?;

        // For wasm-bindgen projects, return JS file for easier loading
        // For standard WASM, return the WASM file
        Ok(result.js_path.unwrap_or(result.wasm_path))
    }
}

impl Default for WasmRustPlugin {
    fn default() -> Self {
        Self::new()
    }
}
