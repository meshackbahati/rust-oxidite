use colored::*;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::output;

/// Find the project root by searching for `Cargo.toml`
fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Run a single Rust file as a script.
///
/// This command supports two modes:
/// 1. **Standalone mode**: Run a Rust file outside any project by creating
///    a cached temporary project. Subsequent runs with the same dependencies
///    reuse the build cache for near-instant execution.
/// 2. **Project mode**: Run a Rust file inside an existing Oxidite/Cargo
///    project by placing it in `src/bin/` and using `cargo run --bin`.
///
/// # Examples
///
/// ```bash
/// # Run a standalone script
/// oxidite run hello.rs
///
/// # Run a script inside a project
/// oxidite run src/bin/migrate_users.rs
///
/// # Run with extra dependencies
/// oxidite run script.rs --deps serde,chrono
/// ```
pub fn run_file(file: &str, extra_deps: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new(file);

    if !file_path.exists() {
        return Err(format!("File not found: {}", file).into());
    }

    if !file_path.extension().map_or(false, |ext| ext == "rs") {
        return Err("File must be a .rs file".into());
    }

    output::header("Oxidite Runner v2.3.3");
    output::debug(&format!("Executing file: {}", file_path.display()));

    // Check if we're inside a Cargo project
    if let Some(project_root) = find_project_root() {
        // Project mode: place the file in src/bin/ and run it
        output::info("Running in project mode");
        output::debug(&format!("Project root: {}", project_root.display()));
        run_in_project(&project_root, file_path, extra_deps)
    } else {
        // Standalone mode: use cached temp project
        output::info("Running in standalone mode");
        run_standalone(file_path, extra_deps)
    }
}

fn run_in_project(
    project_root: &Path,
    file_path: &Path,
    extra_deps: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let src_bin = project_root.join("src").join("bin");
    fs::create_dir_all(&src_bin)?;

    let bin_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Could not determine binary name")?;

    let target_path = src_bin.join(format!("{}.rs", bin_name));

    let is_already_in_bin = fs::canonicalize(file_path)
        .ok()
        .zip(fs::canonicalize(&target_path).ok())
        .map_or(false, |(a, b)| a == b);

    let cleanup_required = if is_already_in_bin {
        false
    } else {
        fs::copy(file_path, &target_path)?;
        output::info(&format!(
            "Copied {} -> {}",
            file_path.display(),
            target_path.display()
        ));
        true
    };

    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_backup = if extra_deps.is_some() {
        let original = fs::read_to_string(&cargo_toml_path).ok();
        if let Some(deps) = extra_deps {
            add_dependencies(project_root, deps)?;
        }
        original
    } else {
        None
    };

    output::step(&format!("Running {} in project context", bin_name.bold()));

    let status = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg(bin_name)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .current_dir(project_root)
        .status()?;

    // Cleanup: remove copied file and restore Cargo.toml
    if cleanup_required {
        let _ = fs::remove_file(&target_path);
        output::debug(&format!("Cleaned up {}", target_path.display()));
    }
    if let Some(original) = cargo_backup {
        let _ = fs::write(&cargo_toml_path, &original);
        output::debug("Restored original Cargo.toml");
    }

    if !status.success() {
        return Err(format!("Script exited with status: {}", status).into());
    }

    output::success("Completed successfully");
    Ok(())
}

fn run_standalone(
    file_path: &Path,
    extra_deps: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut deps: Vec<(String, String)> = vec![
        ("oxidite".into(), "\"2.3.4\"".into()),
        ("tokio".into(), "{ version = \"1\", features = [\"full\"] }".into()),
        // Direct sub-crate deps so `use oxidite_auth::*` style imports work.
        ("oxidite-core".into(),       "\"*\"".into()),
        ("oxidite-middleware".into(), "\"*\"".into()),
        ("oxidite-config".into(),     "\"*\"".into()),
        ("oxidite-db".into(),         "\"*\"".into()),
        ("oxidite-auth".into(),       "\"*\"".into()),
        ("oxidite-queue".into(),      "\"*\"".into()),
        ("oxidite-cache".into(),      "\"*\"".into()),
        ("oxidite-realtime".into(),   "\"*\"".into()),
        ("oxidite-template".into(),   "\"*\"".into()),
        ("oxidite-mail".into(),       "\"*\"".into()),
        ("oxidite-storage".into(),    "\"*\"".into()),
        ("oxidite-security".into(),   "\"*\"".into()),
        ("oxidite-macros".into(),     "\"*\"".into()),
        ("oxidite-utils".into(),      "\"*\"".into()),
        ("oxidite-openapi".into(),    "\"*\"".into()),
        ("oxidite-graphql".into(),    "\"*\"".into()),
        ("oxidite-plugin".into(),     "\"*\"".into()),
        ("serde".into(),              "{ version = \"1\", features = [\"derive\"] }".into()),
        ("serde_json".into(),         "\"1\"".into()),
    ];

    // Parse extra dependencies
    if let Some(extra) = extra_deps {
        for dep in extra.split(',') {
            let dep = dep.trim();
            if !dep.is_empty() {
                // Support "crate@version" syntax, e.g. "serde@1.0" or just "serde"
                if let Some((name, version)) = dep.split_once('@') {
                    deps.push((name.trim().into(), format!("\"{}\"", version.trim())));
                } else {
                    deps.push((dep.into(), "\"*\"".into()));
                }
            }
        }
    }

    // Hash the deps to create a unique cache directory.
    // Same deps = same cache = fast re-runs.
    let deps_hash = {
        let mut hasher = DefaultHasher::new();
        for (name, version) in &deps {
            name.hash(&mut hasher);
            version.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    };

    let cache_dir = dirs_cache().join(&deps_hash);
    let src_dir = cache_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Generate Cargo.toml content
    let cargo_toml_content = format!(
        r#"[package]
name = "oxidite-script"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
        deps.iter()
            .map(|(name, version)| format!("{} = {}", name, version))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Only write Cargo.toml if it changed (avoids cargo re-resolving deps).
    let cargo_toml_path = cache_dir.join("Cargo.toml");
    write_if_changed(&cargo_toml_path, &cargo_toml_content)?;

    // Copy the script to src/main.rs only if it changed.
    let script_content = fs::read_to_string(file_path)?;
    let main_rs_path = src_dir.join("main.rs");
    write_if_changed(&main_rs_path, &script_content)?;

    output::info(&format!("Running {} (standalone)", file_path.display()));
    output::debug(&format!("Cache: {}", cache_dir.display()));

    // Run with --quiet to suppress cargo build output.
    // On first run this compiles (slow). On re-runs with the same deps
    // cargo sees nothing changed and just runs the binary (fast).
    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .current_dir(&cache_dir)
        .status()?;

    if !status.success() {
        return Err(format!("Script exited with status: {}", status).into());
    }

    output::success("Completed successfully");
    Ok(())
}

/// Get the cache base directory for standalone script builds.
fn dirs_cache() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg).join("oxidite-run")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("oxidite-run")
    } else {
        std::env::temp_dir().join("oxidite-run-cache")
    }
}

/// Write content to a file only if the file doesn't exist or its content differs.
/// This preserves mtime so cargo's incremental compilation stays warm.
fn write_if_changed(path: &Path, content: &str) -> Result<bool, std::io::Error> {
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(false); // No change
            }
        }
    }
    fs::write(path, content)?;
    Ok(true) // Changed
}

fn add_dependencies(project_root: &Path, deps: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = project_root.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path)?;

    let mut needs_update = false;

    for dep in deps.split(',') {
        let dep = dep.trim();
        if dep.is_empty() {
            continue;
        }

        // Check if dependency already exists
        if !cargo_toml.contains(&format!("{} = ", dep)) && !cargo_toml.contains(&format!("{}=", dep))
        {
            // Add the dependency under [dependencies]
            if cargo_toml.contains("[dependencies]") {
                let lines: Vec<&str> = cargo_toml.lines().collect();
                let mut new_lines: Vec<String> = Vec::new();
                let mut in_deps = false;
                let mut inserted = false;
                let dep_line = format!("{} = \"*\"", dep);

                for line in &lines {
                    if line.trim().starts_with('[') && in_deps && !inserted {
                        new_lines.push(dep_line.clone());
                        inserted = true;
                        in_deps = false;
                    }
                    if line.trim() == "[dependencies]" {
                        in_deps = true;
                    }
                    new_lines.push(line.to_string());
                }

                if !inserted {
                    new_lines.push(dep_line);
                }

                cargo_toml = new_lines.join("\n");
                needs_update = true;
            }
        }
    }

    if needs_update {
        fs::write(cargo_toml_path, cargo_toml)?;
        output::info(&format!("Added dependencies: {}", deps));
    }

    Ok(())
}
