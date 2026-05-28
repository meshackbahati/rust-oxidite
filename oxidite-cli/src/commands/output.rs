use colored::Colorize;
use std::env;

/// Initialize colored output for the CLI
pub fn init_colors() {
    // Force colors on (can be disabled with NO_COLOR env var)
    colored::control::set_override(true);
}

/// Check if debug mode is enabled
pub fn is_debug() -> bool {
    if let Ok(val) = env::var("OXIDITE_DEBUG") {
        return val == "1" || val.to_lowercase() == "true";
    }
    false
}

/// Print success message in green
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print info message in blue
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print warning message in yellow
#[allow(dead_code)]
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Print error message in red
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg.red());
}

/// Print a header/section title
pub fn header(msg: &str) {
    println!("\n{}", msg.cyan().bold());
}

/// Print a step/action
pub fn step(msg: &str) {
    println!("{} {}", "»".cyan(), msg);
}

/// Format a compile-time error (red, with file:line info)
pub fn compile_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}

/// Format a runtime error (red background, white text)
pub fn runtime_error(msg: &str) {
    eprintln!("{} {}", "runtime error:".red().bold(), msg);
}

/// Format a build failure
pub fn build_failed(msg: &str) {
    eprintln!("\n{}", "Build failed:".red().bold());
    eprintln!("  {}", msg.dimmed());
}

/// Format a successful build
pub fn build_success(msg: &str) {
    println!("\n{}", "Build succeeded:".green().bold());
    println!("  {}", msg);
}

/// Format a command execution result
#[allow(dead_code)]
pub fn command_output(output: &str) {
    println!("{}", output);
}

/// Format a deprecated warning
#[allow(dead_code)]
pub fn deprecated(old: &str, new: &str) {
    warning(&format!(
        "'{}' is deprecated, use '{}' instead",
        old, new
    ));
}

/// Debug log - only prints when OXIDITE_DEBUG=1 or OXIDITE_DEBUG=true
pub fn debug(msg: &str) {
    if is_debug() {
        println!("[{}] {}", "DEBUG".bright_black().bold(), msg.dimmed());
    }
}
