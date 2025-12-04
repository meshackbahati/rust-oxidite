use std::fs;
use std::path::Path;
use chrono::Utc;

/// Create a new migration file
pub fn create_migration(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let migrations_dir = Path::new("migrations");
    
    // Create migrations directory if it doesn't exist
    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir)?;
        println!("✅ Created migrations directory");
    }

    // Generate timestamp-based filename
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_{}.sql", timestamp, name.to_lowercase().replace(' ', "_"));
    let filepath = migrations_dir.join(&filename);

    // Create migration file with template
    let template = format!(
        r#"-- Migration: {}
-- Created at: {}

-- Add migration

-- Add rollback (optional)
"#,
        name,
        Utc::now().to_rfc3339()
    );

    fs::write(&filepath, template)?;
    
    println!("✅ Created migration: {}", filename);
    println!("   Location: {}", filepath.display());
    
    Ok(())
}

/// Run pending migrations
pub async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Running migrations...");
    
    // Check if DATABASE_URL is set
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not set")?;

    // Use sqlx CLI for migrations
    let output = tokio::process::Command::new("sqlx")
        .args(&["migrate", "run", "--database-url", &database_url])
        .output()
        .await?;

    if output.status.success() {
        println!("✅ Migrations completed successfully");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Migration failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Migration failed".into());
    }

    Ok(())
}

/// Revert the last migration
pub async fn revert_migration() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏪ Reverting last migration...");
    
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not set")?;

    let output = tokio::process::Command::new("sqlx")
        .args(&["migrate", "revert", "--database-url", &database_url])
        .output()
        .await?;

    if output.status.success() {
        println!("✅ Migration reverted successfully");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Revert failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Revert failed".into());
    }

    Ok(())
}

/// Show migration status
pub async fn migration_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Migration Status");
    println!("===================\n");

    let migrations_dir = Path::new("migrations");
    
    if !migrations_dir.exists() {
        println!("⚠️  No migrations directory found");
        return Ok(());
    }

    // List all migration files
    let mut migrations: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect();

    migrations.sort();

    if migrations.is_empty() {
        println!("⚠️  No migrations found");
    } else {
        println!("Found {} migration(s):\n", migrations.len());
        for (idx, migration) in migrations.iter().enumerate() {
            println!("  {}. {}", idx + 1, migration);
        }
    }

    Ok(())
}
