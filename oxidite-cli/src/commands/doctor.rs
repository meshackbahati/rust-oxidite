use std::env;
use std::path::Path;
use std::process::Command;

/// Load `.env` from the project root or current directory
fn load_dotenv() {
    // Try to find .env in current directory or project root
    let env_path = Path::new(".env");
    if env_path.exists() {
        let _ = dotenv::from_path(env_path);
        return;
    }

    // Search parent directories for .env
    if let Ok(current) = std::env::current_dir() {
        let mut dir = current;
        while let Some(parent) = dir.parent() {
            let potential_env = parent.join(".env");
            if potential_env.exists() {
                let _ = dotenv::from_path(&potential_env);
                return;
            }
            dir = parent.to_path_buf();
        }
    }

    // Fallback to default dotenv behavior
    let _ = dotenv::dotenv();
}

/// Find project root by searching for Cargo.toml or oxidite.toml in parent directories
fn find_project_root() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").exists() || current.join("oxidite.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file before checking environment variables
    load_dotenv();

    println!("🏥 Oxidite Health Check\n");

    let mut all_ok = true;

    // Check Rust installation
    print!("Checking Rust installation... ");
    match Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        }
        Err(_) => {
            println!("❌ Rust not found");
            all_ok = false;
        }
    }

    // Check Cargo
    print!("Checking Cargo... ");
    match Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        }
        Err(_) => {
            println!("❌ Cargo not found");
            all_ok = false;
        }
    }

    // Check if in an Oxidite project (search parent directories)
    print!("Checking project structure... ");
    if let Some(_root) = find_project_root() {
        println!("✅ Cargo.toml found");
    } else {
        println!("⚠️  Not in a Cargo project directory");
    }

    // Check oxidite.toml or config (search parent directories)
    print!("Checking configuration... ");
    if let Some(root) = find_project_root() {
        if root.join("oxidite.toml").exists() {
            println!("✅ oxidite.toml found");
        } else if root.join("config.toml").exists() {
            println!("✅ config.toml found");
        } else {
            println!("⚠️  No configuration file found (optional)");
        }
    } else {
        println!("⚠️  No configuration file found (optional)");
    }

    // Check migrations directory (search parent directories)
    print!("Checking migrations... ");
    if let Some(root) = find_project_root() {
        let migrations_dir = root.join("migrations");
        if migrations_dir.exists() {
            let count = std::fs::read_dir(&migrations_dir)?.count();
            println!("✅ Found {} migration(s)", count);
        } else {
            println!("ℹ️  No migrations directory");
        }
    } else {
        println!("ℹ️  No migrations directory");
    }

    // Check common dependencies
    println!("\nChecking environment variables:");
    check_env_var("DATABASE_URL");
    check_env_var("REDIS_URL");
    check_env_var("JWT_SECRET");

    println!();
    if all_ok {
        println!("✅ All critical checks passed!");
    } else {
        println!("⚠️  Some checks failed. See above for details.");
    }

    Ok(())
}

fn check_env_var(name: &str) {
    print!("  {}: ", name);
    match env::var(name) {
        Ok(value) => {
            // Mask sensitive values
            let masked = if name.contains("SECRET") || name.contains("PASSWORD") {
                "***"
            } else if value.len() > 30 {
                &format!("{}...", &value[..27])
            } else {
                &value
            };
            println!("✅ {}", masked);
        }
        Err(_) => println!("⚠️  Not set"),
    }
}
