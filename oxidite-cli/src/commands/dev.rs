use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event};
use colored::*;
use std::path::Path;

pub fn start_dev_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🔥 Starting Oxidite development server...".green().bold());
    println!("{}", "👀 Watching for file changes...".cyan());

    // Shared state for the child process
    let child_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    
    // Start initial process
    restart_process(&child_process)?;

    // Setup watcher
    let child_clone = child_process.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    // Debounce logic
    let mut last_restart = Instant::now();
    let debounce_duration = Duration::from_millis(500);

    for res in rx {
        match res {
            Ok(event) => {
                if should_reload(&event) {
                    let now = Instant::now();
                    if now.duration_since(last_restart) > debounce_duration {
                        println!("\n{}", "🔄 Changes detected, restarting...".yellow());
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

fn restart_process(child_lock: &Arc<Mutex<Option<Child>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = child_lock.lock().unwrap();

    // Kill existing process
    if let Some(mut child) = lock.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    // Start new process
    let child = Command::new("cargo")
        .arg("run")
        .spawn();

    match child {
        Ok(c) => {
            *lock = Some(c);
            Ok(())
        },
        Err(e) => {
            println!("{} {}", "❌ Failed to start server:".red(), e);
            Err(Box::new(e))
        }
    }
}

fn should_reload(event: &Event) -> bool {
    for path in &event.paths {
        // Ignore target directory and hidden files
        if path.to_string_lossy().contains("/target/") || 
           path.to_string_lossy().contains("/.") ||
           path.to_string_lossy().contains("\\target\\") {
            return false;
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if matches!(ext_str.as_ref(), "rs" | "toml" | "html" | "css" | "js" | "sql") {
                return true;
            }
        }
    }
    false
}
