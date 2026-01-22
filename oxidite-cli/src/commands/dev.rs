use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use notify::{Watcher, RecursiveMode, Event, event::{ModifyKind, AccessKind}};
use colored::*;
use std::path::Path;
use std::thread;

pub fn start_dev_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Starting Oxidite development server...".green().bold());
    println!("{}", "Watching for file changes...".cyan());

    // Shared state for the child process
    let child_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    
    // Start initial process
    restart_process(&child_process)?;

    // Setup watcher
    let child_clone = child_process.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher = notify::recommended_watcher(move |res| {
        if tx.send(res).is_err() {
            // Channel closed, exit
        }
    })?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    // Debounce logic
    let mut last_restart = Instant::now();
    let debounce_duration = Duration::from_millis(300);

    for res in rx {
        match res {
            Ok(event) => {
                if should_reload(&event) {
                    let now = Instant::now();
                    if now.duration_since(last_restart) > debounce_duration {
                        // Stop current process before restarting
                        stop_process(&child_clone)?;
                        
                        println!("\n{}", "Changes detected, restarting server...".yellow());
                        
                        // Small delay to ensure process is completely stopped
                        thread::sleep(Duration::from_millis(100));
                        
                        restart_process(&child_clone)?;
                        last_restart = now;
                    }
                }
            },
            Err(e) => println!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn stop_process(child_lock: &Arc<Mutex<Option<Child>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = child_lock.lock().unwrap();

    // Kill existing process gracefully first
    if let Some(mut child) = lock.take() {
        // Try graceful termination first
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(())
}

fn restart_process(child_lock: &Arc<Mutex<Option<Child>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = child_lock.lock().unwrap();

    // Kill existing process
    if let Some(mut child) = lock.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    // Start new process - use cargo run with better output handling
    let child = Command::new("cargo")
        .arg("run")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn();

    match child {
        Ok(c) => {
            *lock = Some(c);
            Ok(())
        },
        Err(e) => {
            println!("{} {}", "Failed to start server:".red(), e);
            Err(Box::new(e))
        }
    }
}

fn should_reload(event: &Event) -> bool {
    for path in &event.paths {
        // Ignore target directory and hidden files/directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.starts_with("./target/")
            || path_str.contains("\\target\\")
            || path_str.starts_with(".\\target\\")
            || path_str.contains("/node_modules/")
            || path_str.contains("\\node_modules\\")
            || path_str.starts_with(".") && !path_str.starts_with("./") && !path_str.starts_with(".\\")
        {
            return false;
        }

        // Only react to content modification events, not metadata changes
        match &event.kind {
            notify::EventKind::Modify(ModifyKind::Data(_)) |
            notify::EventKind::Create(_) |
            notify::EventKind::Remove(_) => {},
            _ => return false,
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if matches!(ext_str.as_ref(), "rs" | "toml" | "html" | "css" | "js" | "sql" | "md" | "yaml" | "yml" | "json" | "env") {
                return true;
            }
        }
    }
    false
}