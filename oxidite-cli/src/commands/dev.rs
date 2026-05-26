use std::process::Command;
use crate::env as cli_env;

pub fn run_dev() -> Result<(), Box<dyn std::error::Error>> {
    cli_env::load_env()?;

    println!("Starting Oxidite development server...");

    // In a real implementation this would watch files and restart
    // For now, let's just make sure it passes environment variables

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.envs(std::env::vars());

    let mut child = cmd.spawn()?;
    child.wait()?;

    Ok(())
}
