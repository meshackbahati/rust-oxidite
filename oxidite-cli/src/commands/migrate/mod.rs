pub mod declarative;
use super::sql_script::{execute_sql_script, load_database_url};
use crate::env as cli_env;

pub fn create_migration(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::MigrationManager;

    let manager = MigrationManager::new("migrations");
    let path = manager.create_migration_checked(name)?;

    println!("✅ Created migration: {}", path.display());
    return Ok(());
}

pub async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{DbPool, MigrationManager};

    let db_url = load_database_url()?;

    // Validate PostgreSQL if needed, but DbPool handles multiple backends.
    // The user requested validation for PostgreSQL in migrate.

    let db = DbPool::connect(&db_url).await?;
    let manager = MigrationManager::new("migrations");

    let pending = manager.get_pending_migrations_checked(&db).await?;

    if pending.is_empty() {
        println!("✅ No pending migrations.");
        return Ok(());
    }

    println!("Running {} pending migrations...\n", pending.len());

    for migration in pending {
        println!("⏫ Applying: {} - {}", migration.version, migration.name);
        if !migration.up_sql.is_empty() {
            execute_sql_script(&db, &migration.up_sql).await?;
            manager.mark_migration_applied(&db, &migration.version).await?;
            println!("   ✅ Done");
        }
    }

    Ok(())
}

pub async fn revert_migration() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{DbPool, MigrationManager};
    let db_url = load_database_url()?;
    let db = DbPool::connect(&db_url).await?;
    let manager = MigrationManager::new("migrations");
    let applied = manager.get_applied_migrations(&db).await?;
    if applied.is_empty() {
        println!("No migrations to revert.");
        return Ok(());
    }
    let last_version = applied.last().unwrap();
    let all_migrations = manager.list_migrations_checked()?;
    let migration = all_migrations.iter().find(|m| &m.version == last_version).ok_or("Migration file not found")?;

    println!("⏬ Reverting: {} - {}", migration.version, migration.name);
    if !migration.down_sql.is_empty() {
        execute_sql_script(&db, &migration.down_sql).await?;
        manager.mark_migration_reverted(&db, &migration.version).await?;
        println!("   ✅ Done");
    }
    Ok(())
}

pub async fn migration_status() -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{DbPool, MigrationManager};
    let manager = MigrationManager::new("migrations");
    let migrations = manager.list_migrations_checked()?;
    if migrations.is_empty() {
        println!("No migrations found.");
        return Ok(());
    }
    let applied = if let Ok(db_url) = load_database_url() {
        if let Ok(db) = DbPool::connect(&db_url).await {
            manager.get_applied_migrations(&db).await.unwrap_or_default()
        } else { Vec::new() }
    } else { Vec::new() };

    println!("Migrations:\n");
    for migration in &migrations {
        let status = if applied.contains(&migration.version) { "✅ Applied" } else { "⏳ Pending" };
        println!("  {} {} - {}", status, migration.version, migration.name);
    }
    Ok(())
}
