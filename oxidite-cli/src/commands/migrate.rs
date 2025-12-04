

pub fn create_migration(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::MigrationManager;
    
    let manager = MigrationManager::new("migrations");
    let path = manager.create_migration(name)?;
    
    println!("✅ Created migration: {}", path.display());
    println!("\nEdit the migration file to add SQL:");
    println!("  - Add your UP migration after '-- migrate:up'");
    println!("  - Add your DOWN migration after '-- migrate:down'");
    
    Ok(())
}

pub async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{MigrationManager, DbPool, Database};
    use oxidite_config::Config;
    
    // Load database URL from config
    let config = Config::load()?;
    let db_url = config.get("database.url")
        .unwrap_or("sqlite://data.db".to_string());
    
    let db = DbPool::connect(&db_url).await?;
    let manager = MigrationManager::new("migrations");
    
    let migrations = manager.list_migrations()?;
    
    if migrations.is_empty() {
        println!("No migrations found.");
        return Ok(());
    }
    
    println!("Running {} migrations...\n", migrations.len());
    
    // TODO: Track applied migrations in database
    // For now, just execute all

    for migration in migrations {
        println!("⏫ Applying: {} - {}", migration.version, migration.name);
        
        if !migration.up_sql.is_empty() {
            db.execute(&migration.up_sql).await?;
            println!("   ✅ Done");
        } else {
            println!("   ⚠️  Empty migration");
        }
    }
    
    println!("\n✅ All migrations run successfully!");
    
    Ok(())
}

pub async fn revert_migration() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Revert not yet implemented");
    println!("Manually rollback using the down migration SQL");
    Ok(())
}

pub async fn migration_status() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::MigrationManager;
    
    let manager = MigrationManager::new("migrations");
    let migrations = manager.list_migrations()?;
    
    if migrations.is_empty() {
        println!("No migrations found.");
        return Ok(());
    }
    
    println!("Migrations:\n");
    for migration in &migrations {
        println!("  {} - {}", migration.version, migration.name);
    }
    println!("\nTotal: {} migrations", migrations.len());
    
    Ok(())
}
