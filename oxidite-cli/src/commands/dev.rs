use colored::*;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use notify::{event::ModifyKind, Event, RecursiveMode, Watcher};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use std::env;

use super::output;

/// Find the project root by searching for `oxidite.toml` or `Cargo.toml`
/// starting from the current directory and walking up the tree.
fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("oxidite.toml").exists() || current.join("Cargo.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Load `.env` from the project root, if present.
fn load_dotenv() {
    if env::var("OXIDITE_SKIP_DOTENV").is_err() {
        if let Some(root) = find_project_root() {
            let env_path = root.join(".env");
            if env_path.exists() {
                let _ = dotenv::from_path(&env_path);
                return;
            }
        }
        let _ = dotenv::dotenv();
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub env: Option<String>,
    pub bin: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DevOptions {
    pub run: RunOptions,
    pub watch: Vec<PathBuf>,
    pub ignore: Vec<String>,
    pub hot_reload: bool,
}

pub fn run_project_once(
    release: bool,
    options: &RunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();
    output::debug("Loading environment variables from .env");

    if release {
        output::info("Starting Oxidite server in release mode");
    } else {
        output::info("Starting Oxidite project in debug mode");
    }
    output::debug(&format!("Server options: {:?}", options));

    let mut child = spawn_cargo_run(release, options)?;
    let status = child.wait()?;
    if status.success() {
        output::success("Process completed successfully");
        Ok(())
    } else {
        Err(format!("process exited with status {status}").into())
    }
}

pub fn start_dev_server(options: DevOptions) -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();

    if !options.hot_reload {
        output::warning("Hot reload disabled; running project once");
        return run_project_once(false, &options.run);
    }

    let project_root = find_project_root()
        .ok_or("No Cargo.toml found in current or parent directories")?;

    // Determine binary name and path
    let (bin_name, binary_path) = resolve_binary_path(&project_root, &options.run);
    output::debug(&format!("Binary target: {}", bin_name));
    output::debug(&format!("Binary path: {}", binary_path.display()));

    output::success("Starting Oxidite development server");
    output::info("Watching for file changes");

    // Build once, then start the server
    if !build_project(&project_root, &bin_name)? {
        output::error("Initial build failed");
        std::process::exit(1);
    }

    let child_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    start_binary(&child_process, &binary_path, &options.run)?;

    let watch_paths = if options.watch.is_empty() {
        vec![project_root.clone()]
    } else {
        options.watch.clone()
    };
    let ignore_patterns = default_ignore_patterns(&options.ignore);

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    for path in &watch_paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
        } else {
            println!("⚠️  Watch path not found: {}", path.display());
        }
    }

    // State for the compile-ahead loop
    let should_rebuild: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let build_in_progress: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let mut last_restart = Instant::now();
    let debounce_duration = Duration::from_millis(200);

    for res in rx {
        match res {
            Ok(event) => {
                if should_reload(&event, &ignore_patterns) {
                    let now = Instant::now();
                    if now.duration_since(last_restart) > debounce_duration {
                        *should_rebuild.lock().unwrap() = true;
                        last_restart = now;
                    }
                }
            }
            Err(err) => println!("Watch error: {err:?}"),
        }

        // Try to rebuild if requested and not already building
        let rebuild_requested = *should_rebuild.lock().unwrap();
        let already_building = *build_in_progress.lock().unwrap();

        if rebuild_requested && !already_building {
            *should_rebuild.lock().unwrap() = false;
            *build_in_progress.lock().unwrap() = true;

            println!("\n{}", "Changes detected, rebuilding...".yellow());

            // Spawn a build thread — the old server keeps running
            let root = project_root.clone();
            let bname = bin_name.clone();
            let child_lock = child_process.clone();
            let bpath = binary_path.clone();
            let run_opts = options.run.clone();
            let build_flag = build_in_progress.clone();

            thread::spawn(move || {
                let success = build_project(&root, &bname).unwrap_or(false);
                if success {
                    // Graceful swap: SIGTERM → wait → start new binary
                    graceful_stop(&child_lock);
                    let _ = start_binary(&child_lock, &bpath, &run_opts);
                    println!("{}", "Server restarted with new code.".green());
                } else {
                    println!(
                        "{}",
                        "Build failed — old server is still running.".yellow()
                    );
                }
                *build_flag.lock().unwrap() = false;
            });
        }
    }

    Ok(())
}

/// Build the project binary. Returns true on success.
fn build_project(project_root: &Path, bin_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--bin")
        .arg(bin_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(project_root);

    let mut child = cmd.spawn()?;

    // Stream stderr in real-time so the user sees compilation progress
    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        if let Ok(line) = line {
            eprintln!("{}", line);
        }
    }

    let status = child.wait()?;
    Ok(status.success())
}

/// Start the compiled binary in the background. Returns immediately.
fn start_binary(
    child_lock: &Arc<Mutex<Option<Child>>>,
    binary_path: &Path,
    options: &RunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(binary_path);
    apply_run_env(&mut cmd, options);
    let child = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()?;
    *child_lock.lock().unwrap() = Some(child);
    Ok(())
}

/// Resolve the binary path from project metadata.
fn resolve_binary_path(project_root: &Path, options: &RunOptions) -> (String, PathBuf) {
    let bin_name = options
        .bin
        .clone()
        .unwrap_or_else(|| {
            // Read package name from Cargo.toml
            let cargo_toml = project_root.join("Cargo.toml");
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                for line in content.lines() {
                    if let Some(name) = line.strip_prefix("name = ") {
                        return name
                            .trim_matches('"')
                            .trim()
                            .to_string();
                    }
                }
            }
            "app".to_string()
        });
    let binary_path = project_root.join("target").join("debug").join(&bin_name);
    (bin_name, binary_path)
}

/// Gracefully stop a running process with SIGTERM, then SIGKILL if needed.
fn graceful_stop(child_lock: &Arc<Mutex<Option<Child>>>) {
    let mut lock = child_lock.lock().unwrap();
    if let Some(ref mut child) = *lock {
        let pid = child.id();

        // SIGTERM — allow graceful shutdown for in-flight requests
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);

        // Wait up to 2s for graceful shutdown
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(_)) => {
                    *lock = None;
                    return;
                }
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(_) => break,
            }
        }

        // Force kill if still running
        let _ = child.kill();
        let _ = child.wait();
    }
    *lock = None;
}

/// Spawn `cargo run` (used by `serve` / `run_project_once`).
fn spawn_cargo_run(release: bool, options: &RunOptions) -> std::io::Result<Child> {
    let mut command = Command::new("cargo");
    command.arg("run");
    if release {
        command.arg("--release");
    }
    if let Some(bin) = &options.bin {
        command.arg("--bin").arg(bin);
    }
    apply_run_env(&mut command, options);
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
}

pub fn apply_run_env(command: &mut Command, options: &RunOptions) {
    if let Some(host) = &options.host {
        command.env("SERVER_HOST", host);
    }
    if let Some(port) = options.port {
        command.env("SERVER_PORT", port.to_string());
    }
    if let Some(environment) = &options.env {
        command.env("OXIDITE_ENV", environment);
    }
}

fn should_reload(event: &Event, ignore_patterns: &[String]) -> bool {
    let is_relevant_event = matches!(
        event.kind,
        notify::EventKind::Modify(ModifyKind::Data(_))
            | notify::EventKind::Create(_)
            | notify::EventKind::Remove(_)
    );
    if !is_relevant_event {
        return false;
    }

    for path in &event.paths {
        let path_str = path.to_string_lossy();
        if should_ignore_path(&path_str, ignore_patterns) {
            continue;
        }

        if is_reloadable_path(path) {
            return true;
        }
    }

    false
}

fn should_ignore_path(path: &str, ignore_patterns: &[String]) -> bool {
    if path.contains("/target/")
        || path.contains("\\target\\")
        || path.contains("/node_modules/")
        || path.contains("\\node_modules\\")
        || path.contains("/.git/")
        || path.contains("\\.git\\")
    {
        return true;
    }

    ignore_patterns
        .iter()
        .filter(|pattern| !pattern.is_empty())
        .any(|pattern| path.contains(pattern))
}

fn is_reloadable_path(path: &Path) -> bool {
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        if matches!(
            file_name,
            "Cargo.toml" | "Cargo.lock" | "oxidite.toml" | ".env"
        ) {
            return true;
        }
    }

    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(
            "rs" | "toml" | "html" | "css" | "js" | "sql" | "md" | "yaml" | "yml" | "json" | "env"
        )
    )
}

fn default_ignore_patterns(extra: &[String]) -> Vec<String> {
    let mut patterns = vec![
        "/target/".to_string(),
        "\\target\\".to_string(),
        "/node_modules/".to_string(),
        "\\node_modules\\".to_string(),
        "/.git/".to_string(),
        "\\.git\\".to_string(),
    ];

    for pattern in extra {
        if !pattern.is_empty() && !patterns.contains(pattern) {
            patterns.push(pattern.clone());
        }
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::{default_ignore_patterns, is_reloadable_path, should_ignore_path};
    use std::path::Path;

    #[test]
    fn extends_ignore_patterns() {
        let patterns = default_ignore_patterns(&["dist".to_string()]);
        assert!(patterns.iter().any(|pattern| pattern == "dist"));
        assert!(patterns.iter().any(|pattern| pattern == "/target/"));
    }

    #[test]
    fn ignores_expected_paths() {
        let patterns = default_ignore_patterns(&[]);
        assert!(should_ignore_path("./target/debug/app", &patterns));
        assert!(should_ignore_path(
            "./dist/bundle.js",
            &["dist".to_string()]
        ));
        assert!(!should_ignore_path("./src/main.rs", &patterns));
    }

    #[test]
    fn marks_reloadable_files() {
        assert!(is_reloadable_path(Path::new("src/main.rs")));
        assert!(is_reloadable_path(Path::new("oxidite.toml")));
        assert!(!is_reloadable_path(Path::new("README.txt")));
    }
}
