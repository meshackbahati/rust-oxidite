use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Model, attributes(validate))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    
    // Generate table name: snake_case + 's' (very naive pluralization for now)
    let table_name = name.to_string().to_lowercase() + "s";
    
    // Get fields
    let fields = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    fields.named.iter().collect::<Vec<_>>()
                },
                _ => Vec::new(),
            }
        },
        _ => Vec::new(),
    };
    
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_names_str: Vec<_> = field_names.iter().map(|f| f.to_string()).collect();
    
    // Check for timestamp fields
    let has_created_at = field_names_str.contains(&"created_at".to_string());
    let has_updated_at = field_names_str.contains(&"updated_at".to_string());
    
    // Check for soft delete field
    let has_deleted_at = field_names_str.contains(&"deleted_at".to_string());

    // Filter out 'id' for create/update columns
    // Also filter out timestamps from bind list because we will handle them manually
    let non_id_fields: Vec<_> = fields.iter()
        .filter(|f| {
            let name = f.ident.as_ref().unwrap().to_string();
            name != "id" && name != "created_at" && name != "updated_at" && name != "deleted_at"
        })
        .collect();
        
    let non_id_names: Vec<_> = non_id_fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let non_id_names_str: Vec<_> = non_id_names.iter().map(|f| f.to_string()).collect();
    
    // Create query generation
    let mut create_cols_list = non_id_names_str.clone();
    if has_created_at { create_cols_list.push("created_at".to_string()); }
    if has_updated_at { create_cols_list.push("updated_at".to_string()); }
    // deleted_at is usually null on creation, so we skip it unless we want to support creating deleted records
    
    let create_cols = create_cols_list.join(", ");
    let create_placeholders: Vec<_> = (1..=create_cols_list.len()).map(|i| format!("${}", i)).collect();
    let create_placeholders_str = create_placeholders.join(", ");
    let create_query = format!("INSERT INTO {} ({}) VALUES ({})", table_name, create_cols, create_placeholders_str);
    
    // Update query generation
    let mut update_sets_list = Vec::new();
    for (i, name) in non_id_names_str.iter().enumerate() {
        update_sets_list.push(format!("{} = ${}", name, i + 1));
    }
    
    let mut param_count = non_id_names_str.len();
    if has_updated_at {
        param_count += 1;
        update_sets_list.push(format!("updated_at = ${}", param_count));
    }
    
    let update_sets_str = update_sets_list.join(", ");
    let update_where = format!("WHERE id = ${}", param_count + 1);
    let update_query = format!("UPDATE {} SET {} {}", table_name, update_sets_str, update_where);
    
    // Delete query generation
    let hard_delete_query = format!("DELETE FROM {} WHERE id = $1", table_name);
    
    let delete_impl = if has_deleted_at {
        let soft_delete_query = format!("UPDATE {} SET deleted_at = $1 WHERE id = $2", table_name);
        quote! {
            async fn delete(&self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let now = oxidite_db::chrono::Utc::now().timestamp();
                let query = oxidite_db::sqlx::query(#soft_delete_query)
                    .bind(now)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
        }
    } else {
        quote! {
            async fn delete(&self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let query = oxidite_db::sqlx::query(#hard_delete_query)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
        }
    };

    // Code generation parts for timestamps
    let created_at_logic = if has_created_at {
        quote! {
            let now = oxidite_db::chrono::Utc::now().timestamp();
            self.created_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    let updated_at_create_logic = if has_updated_at {
        quote! {
            let now = oxidite_db::chrono::Utc::now().timestamp();
            self.updated_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    let updated_at_update_logic = if has_updated_at {
        quote! {
            let now = oxidite_db::chrono::Utc::now().timestamp();
            self.updated_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    // Generate validation checks
    let mut validation_checks = Vec::new();
    for field in &fields {
        let field_name = field.ident.as_ref().unwrap();
        for attr in &field.attrs {
            if attr.path().is_ident("validate") {
                let attr_str = attr.to_token_stream().to_string();
                if attr_str.contains("email") {
                    validation_checks.push(quote! {
                        {
                            static EMAIL_REGEX: oxidite_db::once_cell::sync::Lazy<oxidite_db::regex::Regex> = 
                                oxidite_db::once_cell::sync::Lazy::new(|| oxidite_db::regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());
                            if !EMAIL_REGEX.is_match(&self.#field_name) {
                                return Err(format!("Invalid email format for field {}", stringify!(#field_name)));
                            }
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        #[oxidite_db::async_trait]
        impl oxidite_db::Model for #name {
            fn table_name() -> &'static str {
                #table_name
            }
            
            fn fields() -> &'static [&'static str] {
                &[#(#field_names_str),*]
            }
            
            fn has_soft_delete() -> bool {
                #has_deleted_at
            }
            
            async fn create(&mut self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let query = oxidite_db::sqlx::query(#create_query);
                #(
                    let query = query.bind(&self.#non_id_names);
                )*
                #created_at_logic
                #updated_at_create_logic
                
                db.execute_query(query).await?;
                Ok(())
            }
            
            async fn update(&mut self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let query = oxidite_db::sqlx::query(#update_query);
                #(
                    let query = query.bind(&self.#non_id_names);
                )*
                #updated_at_update_logic
                
                let query = query.bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
            
            #delete_impl
            
            async fn force_delete(&self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let query = oxidite_db::sqlx::query(#hard_delete_query)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
            
            fn validate(&self) -> std::result::Result<(), String> {
                #(#validation_checks)*
                Ok(())
            }
        }
    };
    
    TokenStream::from(expanded)
}
