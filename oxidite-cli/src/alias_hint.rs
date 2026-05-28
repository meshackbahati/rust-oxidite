use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static HINT_SHOWN: AtomicBool = AtomicBool::new(false);

/// Get the hint file path in CARGO_HOME
fn get_hint_file() -> PathBuf {
    let cargo_home = env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Fallback to ~/.cargo
            if let Some(home) = env::var_os("HOME") {
                PathBuf::from(home).join(".cargo")
            } else if let Some(home) = env::var_os("USERPROFILE") {
                // Windows fallback
                PathBuf::from(home).join(".cargo")
            } else {
                return PathBuf::new();
            }
        });
    cargo_home.join(".oxidite_alias_hint_shown")
}

/// Print alias suggestion on first run
pub fn print_alias_hint() {
    // Only show once per installation
    let hint_file = get_hint_file();
    if hint_file.exists() || HINT_SHOWN.load(Ordering::Relaxed) {
        return;
    }
    
    HINT_SHOWN.store(true, Ordering::Relaxed);
    let _ = std::fs::write(&hint_file, "shown");
    
    eprintln!("\nTip: Add this alias to your shell config for shorter commands:");
    
    // Detect shell
    let shell = env::var("SHELL").unwrap_or_default();
    if shell.contains("zsh") {
        eprintln!("  echo \"alias oxi='oxidite'\" >> ~/.zshrc");
        eprintln!("  source ~/.zshrc");
    } else if shell.contains("bash") {
        eprintln!("  echo \"alias oxi='oxidite'\" >> ~/.bashrc");
        eprintln!("  source ~/.bashrc");
    } else {
        eprintln!("  # Add to your shell config:");
        eprintln!("  alias oxi='oxidite'");
    }
    
    if cfg!(windows) {
        eprintln!("\nWindows (PowerShell):");
        eprintln!("  Set-Alias oxi oxidite  # Add to $PROFILE");
        eprintln!("\nWindows (CMD):");
        eprintln!("  doskey oxi=oxidite");
    }
    
    eprintln!();
}
