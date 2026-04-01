use colored::*;
use notify::{event::ModifyKind, Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub env: Option<String>,
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
    if release {
        println!(
            "{}",
            "Starting Oxidite server in release mode...".green().bold()
        );
    } else {
        println!("{}", "Starting Oxidite project...".green().bold());
    }

    let mut child = spawn_project_process(release, options)?;
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("process exited with status {status}").into())
    }
}

pub fn start_dev_server(options: DevOptions) -> Result<(), Box<dyn std::error::Error>> {
    if !options.hot_reload {
        println!("{}", "Hot reload disabled; running project once.".yellow());
        return run_project_once(false, &options.run);
    }

    println!(
        "{}",
        "Starting Oxidite development server...".green().bold()
    );
    println!("{}", "Watching for file changes...".cyan());

    let child_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    restart_process(&child_process, &options.run)?;

    let watch_paths = if options.watch.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        options.watch.clone()
    };
    let ignore_patterns = default_ignore_patterns(&options.ignore);

    let child_clone = child_process.clone();
    let run_options = options.run.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    let mut watched_any = false;
    for path in &watch_paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            watched_any = true;
        } else {
            println!("⚠️  Watch path not found: {}", path.display());
        }
    }

    if !watched_any {
        watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    }

    let mut last_restart = Instant::now();
    let debounce_duration = Duration::from_millis(300);

    for res in rx {
        match res {
            Ok(event) => {
                if should_reload(&event, &ignore_patterns) {
                    let now = Instant::now();
                    if now.duration_since(last_restart) > debounce_duration {
                        stop_process(&child_clone)?;
                        println!("\n{}", "Changes detected, restarting server...".yellow());
                        thread::sleep(Duration::from_millis(100));
                        restart_process(&child_clone, &run_options)?;
                        last_restart = now;
                    }
                }
            }
            Err(err) => println!("Watch error: {err:?}"),
        }
    }

    Ok(())
}

fn stop_process(child_lock: &Arc<Mutex<Option<Child>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = child_lock.lock().unwrap();
    if let Some(mut child) = lock.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

fn restart_process(
    child_lock: &Arc<Mutex<Option<Child>>>,
    options: &RunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = child_lock.lock().unwrap();
    if let Some(mut child) = lock.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    let child = spawn_project_process(false, options)?;
    *lock = Some(child);
    Ok(())
}

fn spawn_project_process(release: bool, options: &RunOptions) -> std::io::Result<Child> {
    let mut command = Command::new("cargo");
    command.arg("run");
    if release {
        command.arg("--release");
    }
    apply_run_env(&mut command, options);
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
}

fn apply_run_env(command: &mut Command, options: &RunOptions) {
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
