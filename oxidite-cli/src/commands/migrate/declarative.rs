use std::fs;
use std::path::Path;
use syn::{self, Item, Type, Fields};
use quote::ToTokens;
use oxidite_db::{TableSchema, ColumnSchema, ColumnType};

pub struct ModelScanner {
    models: Vec<TableSchema>,
}

impl ModelScanner {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn scan_directory(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.scan_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn scan_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let file = syn::parse_file(&content)?;

        for item in file.items {
            if let Item::Struct(s) = item {
                let has_model_derive = s.attrs.iter().any(|attr| {
                    attr.path().is_ident("derive") && attr.to_token_stream().to_string().contains("Model")
                });

                if has_model_derive {
                    let table_name = self.extract_table_name(&s);
                    let columns = self.extract_columns(&s);
                    
                    self.models.push(TableSchema {
                        name: table_name,
                        columns,
                    });
                }
            }
        }
        Ok(())
    }

    fn extract_table_name(&self, s: &syn::ItemStruct) -> String {
        for attr in &s.attrs {
            if attr.path().is_ident("model") {
                let attr_str = attr.to_token_stream().to_string();
                // Basic extraction of table_name = "..." or table = "..."
                if let Some(start) = attr_str.find("table_name = \"") {
                    let rest = &attr_str[start + 14..];
                    if let Some(end) = rest.find('"') {
                        return rest[..end].to_string();
                    }
                }
                if let Some(start) = attr_str.find("table = \"") {
                    let rest = &attr_str[start + 9..];
                    if let Some(end) = rest.find('"') {
                        return rest[..end].to_string();
                    }
                }
            }
        }
        // Default: snake_case name + 's'
        format!("{}s", s.ident.to_string().to_lowercase())
    }

    fn extract_columns(&self, s: &syn::ItemStruct) -> Vec<ColumnSchema> {
        let mut columns = Vec::new();
        if let Fields::Named(fields) = &s.fields {
            for field in &fields.named {
                let name = field.ident.as_ref().unwrap().to_string();
                let (ty, nullable) = self.map_type(&field.ty);
                let primary_key = name == "id";
                
                columns.push(ColumnSchema {
                    name,
                    ty,
                    nullable,
                    primary_key,
                    default: None,
                });
            }
        }
        columns
    }

    fn map_type(&self, ty: &Type) -> (ColumnType, bool) {
        let type_str = self.type_to_string(ty);
        
        if type_str.starts_with("Option <") {
            let inner = &type_str[8..type_str.len() - 2];
            let (col_ty, _) = self.map_type_str(inner);
            return (col_ty, true);
        }

        let (col_ty, _) = self.map_type_str(&type_str);
        (col_ty, false)
    }

    fn map_type_str(&self, ty: &str) -> (ColumnType, bool) {
        match ty {
            "i64" => (ColumnType::BigInt, false),
            "i32" => (ColumnType::Int, false),
            "String" => (ColumnType::Text, false),
            "bool" => (ColumnType::Boolean, false),
            "f64" => (ColumnType::Float, false),
            "DateTime < Utc >" | "chrono :: DateTime < chrono :: Utc >" => (ColumnType::DateTime, false),
            "serde_json :: Value" | "Value" => (ColumnType::Json, false),
            "uuid :: Uuid" | "Uuid" => (ColumnType::Uuid, false),
            _ => (ColumnType::Text, false),
        }
    }

    fn type_to_string(&self, ty: &Type) -> String {
        use quote::ToTokens;
        ty.to_token_stream().to_string()
    }

    pub fn models(&self) -> &[TableSchema] {
        &self.models
    }
}

pub async fn make_migrations(name: Option<String>, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    use oxidite_db::{DbPool, MigrationManager, Migration, Database};
    use super::super::sql_script::load_database_url;

    println!("🔍 Scanning models and detecting schema drift...");

    // 1. Scan models in the current project
    let mut scanner = ModelScanner::new();
    if let Err(e) = scanner.scan_directory("src/models") {
        eprintln!("⚠️  Warning: Failed to scan src/models: {}. Trying src/...", e);
        scanner.scan_directory("src")?;
    }
    
    let desired_models = scanner.models();
    if desired_models.is_empty() {
        return Err("No models found with #[derive(Model)]. Ensure your models are in src/models/ and use the derive macro.".into());
    }

    // 2. Connect to the database to inspect current state
    let db_url = load_database_url()?;
    let db = DbPool::connect(&db_url).await?;
    let inspector = db.inspector();
    let db_type = db.db_type();
    
    let mut up_sql = String::new();
    let mut down_sql = String::new();
    
    // 3. Diff each model against the database
    for model in desired_models {
        let current_table = inspector.get_table_schema(&model.name).await?;
        
        if let Some(current) = current_table {
            // Table exists, check for new columns
            for col in &model.columns {
                if !current.columns.iter().any(|c| c.name == col.name) {
                    println!("  ➕ Detected new column: {}.{}", model.name, col.name);
                    
                    let mut col_def = format!("{} {}", col.name, col.ty.to_sql(db_type));
                    if !col.nullable {
                        col_def.push_str(" NOT NULL");
                    }
                    if let Some(def) = &col.default {
                        col_def.push_str(&format!(" DEFAULT {}", def));
                    }
                    
                    up_sql.push_str(&format!("ALTER TABLE {} ADD COLUMN {};\n", model.name, col_def));
                    down_sql.push_str(&format!("ALTER TABLE {} DROP COLUMN {};\n", model.name, col.name));
                }
            }
            
            // Check for dropped columns
            for current_col in &current.columns {
                if !model.columns.iter().any(|c| c.name == current_col.name) {
                    println!("  ➖ Detected removed column: {}.{}", model.name, current_col.name);
                    
                    up_sql.push_str(&format!("ALTER TABLE {} DROP COLUMN {};\n", model.name, current_col.name));
                    
                    // Reconstructing the column for DOWN is hard without full state history, 
                    // so we'll just add a placeholder or skip for now.
                    down_sql.push_str(&format!("-- TODO: Manual rollback for dropped column {}.{}\n", model.name, current_col.name));
                }
            }
        } else {
            // Table doesn't exist, create it
            println!("  🆕 Detected new table: {}", model.name);
            up_sql.push_str(&model.to_create_sql(db_type));
            up_sql.push_str("\n");
            down_sql.push_str(&format!("DROP TABLE {};\n", model.name));
        }
    }

    if up_sql.is_empty() {
        println!("✅ Schema is up to date. No changes detected.");
        return Ok(());
    }

    if dry_run {
        println!("\n🚀 Dry run: SQL to be generated:\n");
        println!("-- migrate:up\n{}", up_sql);
        println!("-- migrate:down\n{}", down_sql);
        return Ok(());
    }

    // 4. Save the generated migration
    let migration_name = name.unwrap_or_else(|| "auto_migration".to_string());
    let _manager = MigrationManager::new("migrations");
    let mut migration = Migration::new(&migration_name);
    migration.up_sql = up_sql;
    migration.down_sql = down_sql;
    
    let path = migration.save("migrations")?;
    println!("\n✅ Generated declarative migration: {}", path.display());
    println!("Run `oxidite migrate run` to apply changes.");
    
    Ok(())
}
