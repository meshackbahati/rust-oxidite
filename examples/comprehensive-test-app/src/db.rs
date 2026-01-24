use oxidite_db::{Database, DbPool, PoolOptions};
use oxidite_core::Error;
use std::sync::Arc;

pub async fn init_db() -> Result<Arc<dyn Database>, Error> {
    println!("Initializing database...");
    
    // In a real app, this would come from config
    let url = "sqlite::memory:";
    
    let mut options = PoolOptions::default();
    options.max_connections = 5;
    options.min_connections = 1;
        
    let db = DbPool::connect_with_options(url, options).await
        .map_err(|e| Error::InternalServerError(format!("Failed to connect to database: {}", e)))?;
        
    println!("Database connected.");
    
    // Run migrations if needed
    // sqlx::migrate!().run(&db.pool).await...
    
    Ok(Arc::new(db))
}
