

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
    
    // Get pending migrations
    let pending = manager.get_pending_migrations(&db).await?;
    
    if pending.is_empty() {
        println!("✅ No pending migrations.");
        return Ok(());
    }
    
    println!("Running {} pending migrations...\n", pending.len());
    
    for migration in pending {
        println!("⏫ Applying: {} - {}", migration.version, migration.name);
        
        if !migration.up_sql.is_empty() {
            db.execute(&migration.up_sql).await?;
            manager.mark_migration_applied(&db, &migration.version).await?;
            println!("   ✅ Done");
        } else {
            println!("   ⚠️  Empty migration");
        }
    }
    
    println!("\n✅ All migrations run successfully!");
    
    Ok(())
}

pub async fn revert_migration() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{MigrationManager, DbPool, Database};
    use oxidite_config::Config;
    
    // Load database URL from config
    let config = Config::load()?;
    let db_url = config.get("database.url")
        .unwrap_or("sqlite://data.db".to_string());
    
    let db = DbPool::connect(&db_url).await?;
    let manager = MigrationManager::new("migrations");
    
    // Get applied migrations
    let applied = manager.get_applied_migrations(&db).await?;
    
    if applied.is_empty() {
        println!("No migrations to revert.");
        return Ok(());
    }
    
    // Get the last applied migration
    let last_version = applied.last().unwrap();
    
    // Find the migration file
    let all_migrations = manager.list_migrations()?;
    let migration = all_migrations
        .iter()
        .find(|m| &m.version == last_version)
        .ok_or("Migration file not found")?;
    
    println!("⏬ Reverting: {} - {}", migration.version, migration.name);
    
    if !migration.down_sql.is_empty() {
        db.execute(&migration.down_sql).await?;
        manager.mark_migration_reverted(&db, &migration.version).await?;
        println!("   ✅ Done");
    } else {
        println!("   ⚠️  No down migration defined");
        return Err("No down migration available".into());
    }
    
    println!("\n✅ Migration reverted successfully!");
    
    Ok(())
}

pub async fn migration_status() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{MigrationManager, DbPool};
    use oxidite_config::Config;
    
    let manager = MigrationManager::new("migrations");
    let migrations = manager.list_migrations()?;
    
    if migrations.is_empty() {
        println!("No migrations found.");
        return Ok(());
    }
    
    // Try to connect to database to get applied migrations
    let applied = if let Ok(config) = Config::load() {
        if let Some(db_url) = config.get("database.url").map(String::from) {
            if let Ok(db) = DbPool::connect(&db_url).await {
                manager.get_applied_migrations(&db).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    println!("Migrations:\n");
    for migration in &migrations {
        let status = if applied.contains(&migration.version) {
            "✅ Applied"
        } else {
            "⏳ Pending"
        };
        println!("  {} {} - {}", status, migration.version, migration.name);
    }
    
    let applied_count = applied.len();
    let pending_count = migrations.len() - applied_count;
    
    println!("\nTotal: {} migrations ({} applied, {} pending)", 
        migrations.len(), applied_count, pending_count);
    
    Ok(())
}
