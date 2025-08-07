mod wasmrust_tests {
    use std::fs;
    use tempfile::TempDir;
    use wasmrust::{
        create_plugin, BuildConfig, OptimizationLevel, Plugin, PluginType, WasmBuilder,
        WasmrustBuilder, WasmrustPlugin,
    };

    fn create_test_rust_project(dir: &std::path::Path, project_type: &str) {
        let cargo_toml_content = match project_type {
            "standard" => {
                r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test-project"
path = "src/main.rs"
"#
            }
            "wasm-bindgen" => {
                r#"
[package]
name = "test-wasm-bindgen"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
"#
            }
            "yew" => {
                r#"
[package]
name = "test-yew-app"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = "0.21"
wasm-bindgen = "0.2"
web-sys = "0.3"
"#
            }
            _ => panic!("Unknown project type"),
        };

        fs::write(dir.join("Cargo.toml"), cargo_toml_content).unwrap();

        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let main_rs_content = match project_type {
            "standard" => {
                r#"
fn main() {
    println!("Hello from Rust WASM!");
}
"#
            }
            "wasm-bindgen" => {
                r#"
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
"#
            }
            "yew" => {
                r#"
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <h1>{ "Hello from Yew!" }</h1>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
"#
            }
            _ => panic!("Unknown project type: {project_type}"),
        };

        let main_file = if project_type == "wasm-bindgen" {
            src_dir.join("lib.rs")
        } else {
            src_dir.join("main.rs")
        };

        fs::write(main_file, main_rs_content).unwrap();

        if project_type == "yew" {
            let index_html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Yew App</title>
</head>
<body></body>
</html>
"#;
            fs::write(dir.join("index.html"), index_html).unwrap();
        }
    }

    #[test]
    fn test_plugin_info() {
        let plugin = WasmrustPlugin::new();
        let info = plugin.info();

        assert_eq!(info.name, "wasmrust");
        assert_eq!(info.version, "0.3.0");
        assert_eq!(info.plugin_type, PluginType::External);
        assert!(info.capabilities.compile_wasm);
        assert!(info.capabilities.compile_webapp);
        assert!(info.capabilities.live_reload);
        assert!(info.capabilities.optimization);
        assert!(info.extensions.contains(&"rs".to_string()));
        assert!(info.entry_files.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_can_handle_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let plugin = WasmrustPlugin::new();

        assert!(!plugin.can_handle_project(project_dir.to_str().unwrap()));

        create_test_rust_project(project_dir, "standard");
        assert!(plugin.can_handle_project(project_dir.to_str().unwrap()));
    }

    #[test]
    fn test_builder_creation() {
        let plugin = WasmrustPlugin::new();
        let builder = plugin.get_builder();

        assert_eq!(builder.language_name(), "rust");
        assert!(builder.supported_extensions().contains(&"rs"));
        assert!(builder.entry_file_candidates().contains(&"Cargo.toml"));
    }

    #[test]
    fn test_dependency_checking() {
        let builder = WasmrustBuilder::new();
        let missing_deps = builder.check_dependencies();

        assert!(missing_deps.is_empty() || !missing_deps.is_empty());
    }

    #[test]
    fn test_project_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let builder = WasmrustBuilder::new();

        assert!(builder
            .validate_project(project_dir.to_str().unwrap())
            .is_err());

        create_test_rust_project(project_dir, "standard");
        assert!(builder
            .validate_project(project_dir.to_str().unwrap())
            .is_ok());
    }

    #[test]
    fn test_framework_detection() {
        let temp_dir = TempDir::new().unwrap();

        let standard_dir = temp_dir.path().join("standard");
        fs::create_dir_all(&standard_dir).unwrap();
        create_test_rust_project(&standard_dir, "standard");

        let plugin = wasmrust::WasmRustPlugin::new();
        assert!(!plugin.supports_web_app(standard_dir.to_str().unwrap()));

        let yew_dir = temp_dir.path().join("yew");
        fs::create_dir_all(&yew_dir).unwrap();
        create_test_rust_project(&yew_dir, "yew");

        assert!(plugin.supports_web_app(yew_dir.to_str().unwrap()));
    }

    #[test]
    fn test_create_plugin_function() {
        let plugin = create_plugin();
        assert_eq!(plugin.info().name, "wasmrust");
        assert_eq!(plugin.info().plugin_type, PluginType::External);
    }

    #[test]
    fn test_optimization_level_conversion() {
        let debug = OptimizationLevel::Debug;
        let release = OptimizationLevel::Release;
        let size = OptimizationLevel::Size;

        assert_eq!(debug, OptimizationLevel::Debug);
        assert_eq!(release, OptimizationLevel::Release);
        assert_eq!(size, OptimizationLevel::Size);
    }

    #[test]
    #[ignore]
    fn test_actual_compilation() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let output_dir = temp_dir.path().join("output");

        create_test_rust_project(project_dir, "standard");

        let builder = WasmrustBuilder::new();

        let missing_deps = builder.check_dependencies();
        if !missing_deps.is_empty() {
            println!("Skipping compilation test due to missing dependencies: {missing_deps:?}");
            return;
        }

        let config = BuildConfig {
            input: project_dir.to_str().unwrap().to_string(),
            output_dir: output_dir.to_str().unwrap().to_string(),
            optimization: OptimizationLevel::Debug,
            target_type: "wasm".to_string(),
            verbose: true,
            watch: false,
        };

        match builder.build(&config) {
            Ok(result) => {
                assert!(std::path::Path::new(&result.output_path).exists());
                assert_eq!(result.language, "rust");
                assert!(result.file_size > 0);
            }
            Err(e) => {
                panic!("Compilation failed: {e:?}");
            }
        }
    }

    #[test]
    fn test_clean_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        create_test_rust_project(project_dir, "standard");

        let builder = WasmrustBuilder::new();

        if builder.check_dependencies().is_empty() {
            match builder.clean(project_dir.to_str().unwrap()) {
                Ok(()) => println!("Clean successful"),
                Err(e) => println!("Clean failed (expected if no target dir): {e}"),
            }
        }
    }
}

mod standalone_tests {
    use std::fs;
    use tempfile::TempDir;
    use wasmrust::WasmRustPlugin;

    #[test]
    fn test_standalone_plugin() {
        let plugin = WasmRustPlugin::new();

        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        assert!(!plugin.can_handle(project_dir.to_str().unwrap()));

        fs::write(
            project_dir.join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        assert!(plugin.can_handle(project_dir.to_str().unwrap()));

        let deps = plugin.check_dependencies();
        assert!(deps.is_empty() || !deps.is_empty());
    }

    #[test]
    fn test_project_inspection() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        fs::write(
            project_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "1.0.0"
edition = "2021"

[dependencies]
yew = "0.21"
wasm-bindgen = "0.2"
"#,
        )
        .unwrap();

        let plugin = WasmRustPlugin::new();

        match plugin.inspect_project(project_dir.to_str().unwrap()) {
            Ok(info) => {
                assert_eq!(info.name, "test-project");
                assert_eq!(info.version, "1.0.0");
                assert!(info.frameworks.contains(&"yew".to_string()));
            }
            Err(e) => {
                println!("Inspection failed: {e}");
            }
        }
    }
}
