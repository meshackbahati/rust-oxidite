use colored::*;
use std::fs;
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
///    a temporary Cargo project.
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

    output::header("Oxidite Runner v2.3.1");
    output::debug(&format!("Executing file: {}", file_path.display()));

    // Check if we're inside a Cargo project
    if let Some(project_root) = find_project_root() {
        // Project mode: place the file in src/bin/ and run it
        output::info("Running in project mode");
        output::debug(&format!("Project root: {}", project_root.display()));
        run_in_project(&project_root, file_path, extra_deps)
    } else {
        // Standalone mode: create a temp project
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

    // Determine the binary name from the file stem
    let bin_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Could not determine binary name")?;

    // If the file is not already in src/bin/, copy it there
    let target_path = src_bin.join(format!("{}.rs", bin_name));
    
    // Check if we need to copy the file (compare canonical paths if both exist)
    let should_copy = if target_path.exists() {
        // Both exist, compare canonical paths
        fs::canonicalize(file_path)? != fs::canonicalize(&target_path)?
    } else {
        // Target doesn't exist yet, definitely need to copy
        true
    };
    
    if should_copy {
        fs::copy(file_path, &target_path)?;
        output::info(&format!("Copied {} -> {}", file_path.display(), target_path.display()));
    }

    // Check if Cargo.toml needs dependencies added
    if let Some(deps) = extra_deps {
        output::debug(&format!("Adding dependencies: {}", deps));
        add_dependencies(project_root, deps)?;
    }

    output::step(&format!("Running {} in project context", bin_name.bold()));

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--bin").arg(bin_name);

    cmd.stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .current_dir(project_root);

    let status = cmd.status()?;

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
    // Create a temporary project directory
    let temp_dir = std::env::temp_dir().join("oxidite_run_temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(temp_dir.join("src"))?;

    // Copy the script to src/main.rs
    fs::copy(file_path, temp_dir.join("src").join("main.rs"))?;

    // Determine dependencies
    let mut deps = vec![
        ("oxidite", "2.3.1"),
        ("oxidite-db", "2.3.1"),
        ("oxidite-config", "2.3.1"),
        ("oxidite-core", "2.3.1"),
        ("tokio", "1"),
    ];

    // Parse extra dependencies
    if let Some(extra) = extra_deps {
        for dep in extra.split(',') {
            let dep = dep.trim();
            if !dep.is_empty() {
                deps.push((dep, "*"));
            }
        }
    }

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "oxidite-script"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
        deps.iter()
            .map(|(name, version)| format!("{} = \"{}\"", name, version))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;

    output::info(&format!("Running {} (standalone mode)", file_path.display()));
    output::debug(&format!("Using temp project at {}", temp_dir.display()));

    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .current_dir(&temp_dir)
        .status()?;

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    if !status.success() {
        return Err(format!("Script exited with status: {}", status).into());
    }

    output::success("Completed successfully");
    Ok(())
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
        if !cargo_toml.contains(&format!("{} = ", dep)) && !cargo_toml.contains(&format!("{}=", dep)) {
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
