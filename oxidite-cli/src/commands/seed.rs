use std::fs;
use std::path::Path;

pub fn create_seeder(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let seeds_dir = Path::new("seeds");
    
    // Create seeds directory if it doesn't exist
    if !seeds_dir.exists() {
        fs::create_dir(seeds_dir)?;
    }
    
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_{}.sql", timestamp, name.to_lowercase());
    let filepath = seeds_dir.join(&filename);
    
    let template = format!(
        r#"-- Seed: {}
-- Created at: {}

-- Add your seed data here
INSERT INTO users (username, email) VALUES ('admin', 'admin@example.com');
"#,
        name,
        chrono::Utc::now().to_rfc3339()
    );
    
    fs::write(&filepath, template)?;
    
    println!("✅ Created seeder: {}", filepath.display());
    println!("\nEdit the file to add your seed data.");
    
    Ok(())
}

pub async fn run_seeders() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{DbPool, Database};
    use oxidite_config::Config;
    
    // Load database URL from config
    let config = Config::load()?;
    let db_url = config.get("database.url")
        .unwrap_or("sqlite://data.db".to_string());
    
    let db = DbPool::connect(&db_url).await?;
    
    let seeds_dir = Path::new("seeds");
    
    if !seeds_dir.exists() {
        println!("No seeds directory found.");
        return Ok(());
    }
    
    // Get all seed files
    let mut seed_files: Vec<_> = fs::read_dir(seeds_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "sql")
                .unwrap_or(false)
        })
        .collect();
    
    // Sort by filename (timestamp)
    seed_files.sort_by_key(|entry| entry.file_name());
    
    if seed_files.is_empty() {
        println!("No seed files found.");
        return Ok(());
    }
    
    println!("Running {} seeders...\n", seed_files.len());
    
    for entry in seed_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        
        println!("🌱 Seeding: {}", filename);
        
        let sql = fs::read_to_string(&path)?;
        
        if !sql.trim().is_empty() {
            // Split by semicolons and execute each statement
            for statement in sql.split(';') {
                let statement = statement.trim();
                if !statement.is_empty() && !statement.starts_with("--") {
                    db.execute(statement).await?;
                }
            }
            println!("   ✅ Done");
        } else {
            println!("   ⚠️  Empty seeder");
        }
    }
    
    println!("\n✅ All seeders run successfully!");
    
    Ok(())
}
