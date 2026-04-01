use std::env;
use std::process::Command;

pub fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
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

    // Check if in an Oxidite project
    print!("Checking project structure... ");
    if std::path::Path::new("Cargo.toml").exists() {
        println!("✅ Cargo.toml found");
    } else {
        println!("⚠️  Not in a Cargo project directory");
    }

    // Check oxidite.toml or config
    print!("Checking configuration... ");
    if std::path::Path::new("oxidite.toml").exists() {
        println!("✅ oxidite.toml found");
    } else if std::path::Path::new("config.toml").exists() {
        println!("✅ config.toml found");
    } else {
        println!("⚠️  No configuration file found (optional)");
    }

    // Check migrations directory
    print!("Checking migrations... ");
    if std::path::Path::new("migrations").exists() {
        let count = std::fs::read_dir("migrations")?.count();
        println!("✅ Found {} migration(s)", count);
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
