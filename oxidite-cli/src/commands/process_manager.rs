use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::{Command, Stdio};

use super::output;

const PROCFILE: &str = ".oxidite_procs.json";

#[derive(Debug, Serialize, Deserialize)]
struct ProcessInfo {
    id: String,
    name: String,
    pid: u32,
    status: String,
    started_at: String,
    cwd: String,
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessList {
    processes: Vec<ProcessInfo>,
}

impl ProcessList {
    fn load() -> Self {
        if let Ok(content) = fs::read_to_string(PROCFILE) {
            if let Ok(list) = serde_json::from_str(&content) {
                return list;
            }
        }
        ProcessList {
            processes: Vec::new(),
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(PROCFILE, content)?;
        Ok(())
    }

    fn add(&mut self, proc: ProcessInfo) {
        self.processes.push(proc);
    }

    fn remove(&mut self, id: &str) -> bool {
        let len = self.processes.len();
        self.processes.retain(|p| p.id != id);
        self.processes.len() < len
    }

    #[allow(dead_code)]
    fn find(&self, id: &str) -> Option<&ProcessInfo> {
        self.processes.iter().find(|p| p.id == id)
    }
}

/// Start a process in the background (PM2-style)
pub fn start_process(name: Option<String>, release: bool) -> Result<(), Box<dyn std::error::Error>> {
    output::header("Starting Oxidite process");

    let proc_name = name.unwrap_or_else(|| "oxidite-app".to_string());
    let proc_id = format!("{}-{}", proc_name, std::process::id());

    // Load process list
    let mut procs = ProcessList::load();

    // Check if process with same name already exists
    if procs.processes.iter().any(|p| p.name == proc_name) {
        output::warning(&format!("Process '{}' already exists, stopping it first", proc_name));
        stop_process_by_name(&proc_name)?;
        procs = ProcessList::load(); // Reload after stop
    }

    output::step(&format!("Starting process '{}'", proc_name));
    output::debug(&format!("Process ID: {}", proc_id));
    output::debug(&format!("Release mode: {}", release));

    // Spawn the process
    let child = Command::new("cargo")
        .arg("run")
        .args(if release { vec!["--release"] } else { vec![] })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();
    let started_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let cwd = std::env::current_dir()?.display().to_string();

    output::info(&format!("Process started with PID {}", pid));

    // Add to process list
    procs.add(ProcessInfo {
        id: proc_id.clone(),
        name: proc_name.clone(),
        pid,
        status: "online".to_string(),
        started_at,
        cwd,
        command: format!("cargo run{}", if release { " --release" } else { "" }),
    });

    procs.save()?;

    output::success(&format!("Process '{}' is now running", proc_name));
    output::debug(&format!("Process file: {}", PROCFILE));

    Ok(())
}

/// Stop a running process by name or ID
pub fn stop_process(identifier: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    output::header("Stopping Oxidite process");

    let procs = ProcessList::load();

    if procs.processes.is_empty() {
        output::warning("No processes found");
        return Ok(());
    }

    let target = if let Some(id) = identifier {
        // Try to find by name or ID
        procs.processes.iter().find(|p| p.name == id || p.id == id)
            .ok_or_else(|| format!("Process '{}' not found", id))?
    } else {
        // Stop all processes
        output::info("Stopping all processes");
        for proc_info in &procs.processes {
            stop_single_process(proc_info)?;
        }
        return Ok(());
    };

    stop_single_process(target)?;
    Ok(())
}

fn stop_single_process(proc_info: &ProcessInfo) -> Result<(), Box<dyn std::error::Error>> {
    output::step(&format!("Stopping process '{}' (PID: {})", proc_info.name, proc_info.pid));

    // Try to kill the process
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        let _ = kill(Pid::from_raw(proc_info.pid as i32), Signal::SIGTERM);
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(&["/F", "/PID", &proc_info.pid.to_string()])
            .output()?;
    }

    // Remove from process list
    let mut procs = ProcessList::load();
    procs.remove(&proc_info.id);
    procs.save()?;

    output::success(&format!("Process '{}' stopped", proc_info.name));
    Ok(())
}

fn stop_process_by_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let procs = ProcessList::load();
    if let Some(proc_info) = procs.processes.iter().find(|p| p.name == name) {
        stop_single_process(proc_info)?;
    }
    Ok(())
}

/// Restart a process
pub fn restart_process(identifier: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    output::header("Restarting Oxidite process");

    let procs = ProcessList::load();

    if procs.processes.is_empty() {
        output::warning("No processes found to restart");
        return Ok(());
    }

    for proc_info in &procs.processes {
        if let Some(ref id) = identifier {
            if proc_info.name != *id && proc_info.id != *id {
                continue;
            }
        }

        output::step(&format!("Restarting '{}'", proc_info.name));

        // Stop the process
        let _ = stop_single_process(proc_info);

        // Start it again
        let is_release = proc_info.command.contains("--release");
        let _ = start_process(Some(proc_info.name.clone()), is_release);
    }

    Ok(())
}

/// List all running processes
pub fn list_processes() -> Result<(), Box<dyn std::error::Error>> {
    output::header("Oxidite Process List");

    let procs = ProcessList::load();

    if procs.processes.is_empty() {
        output::info("No processes running");
        output::info("Start a process with: oxidite pm2 start");
        return Ok(());
    }

    println!("{}", format!("{:<20} {:<8} {:<10} {:<22}", "Name", "PID", "Status", "Started").cyan().bold());
    println!("{}", "─".repeat(62).dimmed());

    for proc_info in &procs.processes {
        let status_color = match proc_info.status.as_str() {
            "online" => proc_info.status.green(),
            "stopped" => proc_info.status.red(),
            "error" => proc_info.status.red().bold(),
            _ => proc_info.status.yellow(),
        };

        println!(
            "{:<20} {:<8} {:<10} {:<22}",
            proc_info.name.bold(),
            proc_info.pid,
            status_color,
            proc_info.started_at
        );
    }

    println!("\nTotal: {} process{}", procs.processes.len(), if procs.processes.len() == 1 { "" } else { "s" });
    Ok(())
}

/// Show detailed info about a specific process
pub fn show_process(identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
    let procs = ProcessList::load();

    let proc_info = procs.processes.iter().find(|p| p.name == identifier || p.id == identifier)
        .ok_or_else(|| format!("Process '{}' not found", identifier))?;

    output::header(&format!("Process: {}", proc_info.name));
    println!("  {:<15} {}", "ID:".bold(), proc_info.id);
    println!("  {:<15} {}", "Name:".bold(), proc_info.name);
    println!("  {:<15} {}", "PID:".bold(), proc_info.pid);
    println!("  {:<15} {}", "Status:".bold(), proc_info.status);
    println!("  {:<15} {}", "Started:".bold(), proc_info.started_at);
    println!("  {:<15} {}", "Working Dir:".bold(), proc_info.cwd);
    println!("  {:<15} {}", "Command:".bold(), proc_info.command);

    Ok(())
}

/// Monitor processes (continuous status updates)
pub fn monitor_processes() -> Result<(), Box<dyn std::error::Error>> {
    output::header("Monitoring Oxidite Processes (Ctrl+C to exit)");

    loop {
        let procs = ProcessList::load();

        if procs.processes.is_empty() {
            output::info("No processes running");
        } else {
            for proc_info in &procs.processes {
                // Check if process is still alive
                let is_alive = is_process_alive(proc_info.pid);

                let status_symbol = if is_alive {
                    "●".green()
                } else {
                    "●".red()
                };

                println!("{} {} (PID: {})", status_symbol, proc_info.name, proc_info.pid);
            }
        }

        println!();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
    }

    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}
