use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitStr, Type,
};

#[proc_macro_derive(Model, attributes(validate, model, has_many, has_one, belongs_to))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match derive_model_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_model_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let default_table_name = format!("{}s", name.to_string().to_lowercase());
    let table_name = parse_table_name(input)?.unwrap_or(default_table_name);

    let named_fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    data.fields.span(),
                    "Model derive requires a struct with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new(
                    data.fields.span(),
                    "Model derive requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "Model derive can only be used on structs",
            ));
        }
    };

    let field_names: Vec<_> = named_fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();
    let field_names_str: Vec<_> = field_names.iter().map(|f| f.to_string()).collect();

    let find_field = |name: &str| {
        named_fields
            .iter()
            .find(|field| field.ident.as_ref().map(|id| id == name).unwrap_or(false))
    };

    if !field_names_str.iter().any(|f| f == "id") {
        return Err(syn::Error::new(
            input.span(),
            "Model derive requires an `id` field",
        ));
    }

    let has_created_at = field_names_str.iter().any(|f| f == "created_at");
    let has_updated_at = field_names_str.iter().any(|f| f == "updated_at");
    let has_deleted_at = field_names_str.iter().any(|f| f == "deleted_at");

    let id_type = if let Some(id_field) = find_field("id") {
        if is_i64_type(&id_field.ty) {
            quote!(i64)
        } else if is_type(&id_field.ty, "Uuid") {
            quote!(::oxidite::db::sqlx::types::Uuid)
        } else if is_string_type(&id_field.ty) {
            quote!(String)
        } else {
            return Err(syn::Error::new(
                id_field.ty.span(),
                "Model derive requires `id` to be of type i64, Uuid, or String",
            ));
        }
    } else {
        quote!(i64)
    };

    if has_created_at {
        let field = find_field("created_at")
            .ok_or_else(|| syn::Error::new(input.span(), "missing `created_at` field"))?;
        if !is_i64_type(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "`created_at` must be i64 for automatic timestamp support",
            ));
        }
    }

    if has_updated_at {
        let field = find_field("updated_at")
            .ok_or_else(|| syn::Error::new(input.span(), "missing `updated_at` field"))?;
        if !is_i64_type(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "`updated_at` must be i64 for automatic timestamp support",
            ));
        }
    }

    if has_deleted_at {
        let field = find_field("deleted_at")
            .ok_or_else(|| syn::Error::new(input.span(), "missing `deleted_at` field"))?;
        if !is_option_i64_type(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "`deleted_at` must be Option<i64> for soft-delete support",
            ));
        }
    }

    let non_id_fields: Vec<_> = named_fields
        .iter()
        .filter(|f| {
            let field_name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
            !matches!(
                field_name.as_str(),
                "id" | "created_at" | "updated_at" | "deleted_at"
            )
        })
        .collect();

    let non_id_names: Vec<_> = non_id_fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();
    let non_id_names_str: Vec<_> = non_id_names.iter().map(|f| f.to_string()).collect();

    let mut create_cols_list = non_id_names_str.clone();
    if has_created_at {
        create_cols_list.push("created_at".to_string());
    }
    if has_updated_at {
        create_cols_list.push("updated_at".to_string());
    }

    let create_query = if create_cols_list.is_empty() {
        format!("INSERT INTO {} DEFAULT VALUES", table_name)
    } else {
        let create_cols = create_cols_list.join(", ");
        let create_placeholders: Vec<_> = (1..=create_cols_list.len())
            .map(|i| format!("${}", i))
            .collect();
        let create_placeholders_str = create_placeholders.join(", ");
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name, create_cols, create_placeholders_str
        )
    };

    let mut update_sets_list = Vec::new();
    for (i, field_name) in non_id_names_str.iter().enumerate() {
        update_sets_list.push(format!("{} = ${}", field_name, i + 1));
    }

    let mut param_count = non_id_names_str.len();
    if has_updated_at {
        param_count += 1;
        update_sets_list.push(format!("updated_at = ${}", param_count));
    }

    let update_sets_str = update_sets_list.join(", ");
    let update_where = format!("WHERE id = ${}", param_count + 1);
    let update_query = format!("UPDATE {} SET {} {}", table_name, update_sets_str, update_where);

    let hard_delete_query = format!("DELETE FROM {} WHERE id = $1", table_name);

    let delete_impl = if has_deleted_at {
        let soft_delete_query = format!("UPDATE {} SET deleted_at = $1 WHERE id = $2", table_name);
        quote! {
            async fn delete(&self, db: &impl oxidite_db::Database) -> oxidite_db::Result<()> {
                let now = oxidite_db::chrono::Utc::now().timestamp();
                let query = ::oxidite::db::sqlx::query(#soft_delete_query)
                    .bind(now)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
        }
    } else {
        quote! {
            async fn delete(&self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<()> {
                let query = ::oxidite::db::sqlx::query(#hard_delete_query)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }
        }
    };

    let created_at_logic = if has_created_at {
        quote! {
            let now = ::oxidite::db::chrono::Utc::now().timestamp();
            self.created_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    let updated_at_create_logic = if has_updated_at {
        quote! {
            let now = ::oxidite::db::chrono::Utc::now().timestamp();
            self.updated_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    let updated_at_update_logic = if has_updated_at {
        quote! {
            let now = ::oxidite::db::chrono::Utc::now().timestamp();
            self.updated_at = now;
            let query = query.bind(now);
        }
    } else {
        quote! {}
    };

    let mut validation_checks = Vec::new();
    for field in &named_fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.span(), "Expected named field"))?;

        for attr in &field.attrs {
            if attr.path().is_ident("validate") {
                let res = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("email") {
                        if !is_string_type(&field.ty) {
                            return Err(meta.error("#[validate(email)] can only be used on String fields"));
                        }
                        validation_checks.push(quote! {
                            {
                                static EMAIL_REGEX: ::oxidite::db::once_cell::sync::Lazy<::oxidite::db::regex::Regex> =
                                    ::oxidite::db::once_cell::sync::Lazy::new(|| ::oxidite::db::regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());
                                if !EMAIL_REGEX.is_match(&self.#field_name) {
                                    return Err(format!("Invalid email format for field {}", stringify!(#field_name)));
                                }
                            }
                        });
                        Ok(())
                    } else if meta.path.is_ident("url") {
                        if !is_string_type(&field.ty) {
                            return Err(meta.error("#[validate(url)] can only be used on String fields"));
                        }
                        validation_checks.push(quote! {
                            {
                                static URL_REGEX: ::oxidite::db::once_cell::sync::Lazy<::oxidite::db::regex::Regex> =
                                    ::oxidite::db::once_cell::sync::Lazy::new(|| ::oxidite::db::regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());
                                if !URL_REGEX.is_match(&self.#field_name) {
                                    return Err(format!("Invalid URL format for field {}", stringify!(#field_name)));
                                }
                            }
                        });
                        Ok(())
                    } else if meta.path.is_ident("length") {
                        if !is_string_type(&field.ty) {
                            return Err(meta.error("#[validate(length)] can only be used on String fields"));
                        }
                        
                        let mut min = None;
                        let mut max = None;
                        
                        meta.parse_nested_meta(|inner| {
                            if inner.path.is_ident("min") {
                                let v: syn::LitInt = inner.value()?.parse()?;
                                min = Some(v.base10_parse::<usize>()?);
                                Ok(())
                            } else if inner.path.is_ident("max") {
                                let v: syn::LitInt = inner.value()?.parse()?;
                                max = Some(v.base10_parse::<usize>()?);
                                Ok(())
                            } else {
                                Err(inner.error("unsupported length property"))
                            }
                        })?;
                        
                        if let Some(min_val) = min {
                            validation_checks.push(quote! {
                                if self.#field_name.len() < #min_val {
                                    return Err(format!("Field {} must be at least {} characters long", stringify!(#field_name), #min_val));
                                }
                            });
                        }
                        if let Some(max_val) = max {
                            validation_checks.push(quote! {
                                if self.#field_name.len() > #max_val {
                                    return Err(format!("Field {} must be at most {} characters long", stringify!(#field_name), #max_val));
                                }
                            });
                        }
                        Ok(())
                    } else if meta.path.is_ident("range") {
                        let mut min = None;
                        let mut max = None;
                        
                        meta.parse_nested_meta(|inner| {
                            if inner.path.is_ident("min") {
                                let v: syn::LitInt = inner.value()?.parse()?;
                                min = Some(v.base10_parse::<i64>()?);
                                Ok(())
                            } else if inner.path.is_ident("max") {
                                let v: syn::LitInt = inner.value()?.parse()?;
                                max = Some(v.base10_parse::<i64>()?);
                                Ok(())
                            } else {
                                Err(inner.error("unsupported range property"))
                            }
                        })?;
                        
                        if let Some(min_val) = min {
                            validation_checks.push(quote! {
                                if (self.#field_name as i64) < #min_val {
                                    return Err(format!("Field {} must be >= {}", stringify!(#field_name), #min_val));
                                }
                            });
                        }
                        if let Some(max_val) = max {
                            validation_checks.push(quote! {
                                if (self.#field_name as i64) > #max_val {
                                    return Err(format!("Field {} must be <= {}", stringify!(#field_name), #max_val));
                                }
                            });
                        }
                        Ok(())
                    } else if meta.path.is_ident("regex") {
                        let value = meta.value()?;
                        let pattern: syn::LitStr = value.parse()?;
                        let pat_str = pattern.value();
                        
                        validation_checks.push(quote! {
                            {
                                static REGEX: ::oxidite::db::once_cell::sync::Lazy<::oxidite::db::regex::Regex> =
                                    ::oxidite::db::once_cell::sync::Lazy::new(|| ::oxidite::db::regex::Regex::new(#pat_str).unwrap());
                                if !REGEX.is_match(&self.#field_name) {
                                    return Err(format!("Field {} does not match required pattern", stringify!(#field_name)));
                                }
                            }
                        });
                        Ok(())
                    } else if meta.path.is_ident("custom") {
                        let value = meta.value()?;
                        let func_str: syn::LitStr = value.parse()?;
                        let func_ident = syn::Ident::new(&func_str.value(), func_str.span());
                        
                        validation_checks.push(quote! {
                            if let Err(e) = self.#func_ident(db).await {
                                return Err(e);
                            }
                        });
                        Ok(())
                    } else if meta.path.is_ident("unique") {
                        let mut table = None;
                        let mut column = None;
                        
                        meta.parse_nested_meta(|inner| {
                            if inner.path.is_ident("table") {
                                let v: syn::LitStr = inner.value()?.parse()?;
                                table = Some(v.value());
                                Ok(())
                            } else if inner.path.is_ident("column") {
                                let v: syn::LitStr = inner.value()?.parse()?;
                                column = Some(v.value());
                                Ok(())
                            } else {
                                Err(inner.error("unsupported unique property"))
                            }
                        })?;
                        
                        let t_name = table.unwrap_or_else(|| table_name.clone());
                        let c_name = column.unwrap_or_else(|| field_name.to_string());
                        
                        validation_checks.push(quote! {
                            {
                                let query = format!("SELECT COUNT(*) FROM {} WHERE {} = $1 AND id != $2", #t_name, #c_name);
                                let row = db.fetch_one(::oxidite::db::sqlx::query(&query).bind(&self.#field_name).bind(self.id)).await
                                    .map_err(|e| format!("Database error during unique validation: {}", e))?;
                                    
                                if let Some(r) = row {
                                    use ::oxidite::db::sqlx::Row;
                                    let count: i64 = r.try_get(0).unwrap_or(0);
                                    if count > 0 {
                                        return Err(format!("Field {} must be unique", stringify!(#field_name)));
                                    }
                                }
                            }
                        });
                        Ok(())
                    } else {
                        // ignore unknown validations instead of erroring, so we don't break backward compatibility
                        Ok(())
                    }
                });
                
                if let Err(e) = res {
                    return Err(e);
                }
            }
        }
    }

    let column_schemas = named_fields.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let (ty, nullable) = map_rust_type_to_column_type(&f.ty);
        let primary_key = field_name == "id";
        
        quote! {
            ::oxidite::db::ColumnSchema {
                name: #field_name.to_string(),
                ty: ::oxidite::db::ColumnType::#ty,
                nullable: #nullable,
                primary_key: #primary_key,
                default: None,
            }
        }
    });

    let is_persisted_logic = if id_type.to_string().contains("i64") {
        quote! { self.id > 0 }
    } else if id_type.to_string().contains("Uuid") {
        quote! { !self.id.is_nil() }
    } else {
        quote! { !self.id.is_empty() }
    };

    let expanded = quote! {
        #[::oxidite::db::async_trait]
        impl ::oxidite::db::Model for #name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn schema() -> ::oxidite::db::TableSchema {
                ::oxidite::db::TableSchema {
                    name: #table_name.to_string(),
                    columns: vec![
                        #(#column_schemas),*
                    ],
                }
            }

            fn fields() -> &'static [&'static str] {
                &[#(#field_names_str),*]
            }

            fn has_soft_delete() -> bool {
                #has_deleted_at
            }

            async fn create(&mut self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<()> {
                let query = ::oxidite::db::sqlx::query(#create_query);
                #(
                    let query = query.bind(&self.#non_id_names);
                )*
                #created_at_logic
                #updated_at_create_logic

                db.execute_query(query).await?;
                Ok(())
            }

            async fn update(&mut self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<()> {
                let query = ::oxidite::db::sqlx::query(#update_query);
                #(
                    let query = query.bind(&self.#non_id_names);
                )*
                #updated_at_update_logic

                let query = query.bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }

            #delete_impl

            async fn force_delete(&self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<()> {
                let query = ::oxidite::db::sqlx::query(#hard_delete_query)
                    .bind(&self.id);
                db.execute_query(query).await?;
                Ok(())
            }

            async fn validate(&self, db: &impl ::oxidite::db::Database) -> std::result::Result<(), String> {
                let _ = db; // silence unused warning if there are no db validations
                #(#validation_checks)*
                Ok(())
            }

            fn is_persisted(&self) -> bool {
                #is_persisted_logic
            }
        }
    };

    let mut relation_methods = Vec::new();
    
    for attr in &input.attrs {
        if attr.path().is_ident("has_many") || attr.path().is_ident("has_one") || attr.path().is_ident("belongs_to") {
            let mut model_name = None;
            let mut foreign_key = None;
            let mut name = None;
            
            let is_has_many = attr.path().is_ident("has_many");
            let is_has_one = attr.path().is_ident("has_one");
            let is_belongs_to = attr.path().is_ident("belongs_to");
            
            let res = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("model") {
                    let v: syn::LitStr = meta.value()?.parse()?;
                    model_name = Some(v.value());
                    Ok(())
                } else if meta.path.is_ident("foreign_key") {
                    let v: syn::LitStr = meta.value()?.parse()?;
                    foreign_key = Some(v.value());
                    Ok(())
                } else if meta.path.is_ident("name") {
                    let v: syn::LitStr = meta.value()?.parse()?;
                    name = Some(v.value());
                    Ok(())
                } else {
                    Err(meta.error("unsupported relation property"))
                }
            });
            
            if let Err(e) = res {
                return Err(e);
            }
            
            let m_name = model_name.ok_or_else(|| syn::Error::new(attr.span(), "missing `model` in relation"))?;
            let fk_name = foreign_key.ok_or_else(|| syn::Error::new(attr.span(), "missing `foreign_key` in relation"))?;
            let rel_name = name.ok_or_else(|| syn::Error::new(attr.span(), "missing `name` in relation"))?;
            
            let model_ident = syn::Ident::new(&m_name, proc_macro2::Span::call_site());
            let rel_ident = syn::Ident::new(&rel_name, proc_macro2::Span::call_site());
            
            if is_has_many {
                let eager_ident = syn::Ident::new(&format!("eager_load_{}", rel_name), proc_macro2::Span::call_site());
                relation_methods.push(quote! {
                    /// Lazy-load: fetches related rows one-by-one (N+1).
                    pub async fn #rel_ident(&self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<Vec<#model_ident>> {
                        #model_ident::query().filter_eq(#fk_name, self.id).fetch_all(db).await
                    }

                    /// Eager-load: fetches related rows for many parents in a single IN query.
                    /// Returns a HashMap keyed by parent id.
                    pub async fn #eager_ident(
                        db: &impl ::oxidite::db::Database,
                        parents: &[Self],
                    ) -> ::oxidite::db::Result<std::collections::HashMap<i64, Vec<#model_ident>>> {
                        let ids: Vec<i64> = parents.iter().map(|p| p.id).collect();
                        ::oxidite::db::HasMany::<Self, #model_ident>::eager_load(db, &ids, #fk_name).await
                    }
                });
            } else if is_has_one {
                let eager_ident = syn::Ident::new(&format!("eager_load_{}", rel_name), proc_macro2::Span::call_site());
                relation_methods.push(quote! {
                    /// Lazy-load: fetches related row one-by-one (N+1).
                    pub async fn #rel_ident(&self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<Option<#model_ident>> {
                        #model_ident::query().filter_eq(#fk_name, self.id).fetch_one(db).await
                    }

                    /// Eager-load: fetches related rows for many parents in a single IN query.
                    /// Returns a HashMap keyed by parent id.
                    pub async fn #eager_ident(
                        db: &impl ::oxidite::db::Database,
                        parents: &[Self],
                    ) -> ::oxidite::db::Result<std::collections::HashMap<i64, Option<#model_ident>>> {
                        let ids: Vec<i64> = parents.iter().map(|p| p.id).collect();
                        ::oxidite::db::HasOne::<Self, #model_ident>::eager_load(db, &ids, #fk_name).await
                    }
                });
            } else if is_belongs_to {
                let fk_ident = syn::Ident::new(&fk_name, proc_macro2::Span::call_site());
                relation_methods.push(quote! {
                    pub async fn #rel_ident(&self, db: &impl ::oxidite::db::Database) -> ::oxidite::db::Result<Option<#model_ident>> {
                        #model_ident::find(db, self.#fk_ident).await
                    }
                });
            }
        }
    }

    let final_expanded = quote! {
        #expanded
        
        impl #name {
            #(#relation_methods)*
        }
    };

    Ok(final_expanded)
}

fn parse_table_name(input: &DeriveInput) -> syn::Result<Option<String>> {
    let mut table_name = None;
    let mut table_alias = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("model") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                if table_name.is_some() {
                    return Err(meta.error("duplicate `table_name` in #[model(...)]"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                table_name = Some(lit.value());
                return Ok(());
            }

            if meta.path.is_ident("table") {
                if table_alias.is_some() {
                    return Err(meta.error("duplicate `table` in #[model(...)]"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                table_alias = Some(lit.value());
                return Ok(());
            }

            Err(meta.error(
                "unsupported model attribute; expected `table_name = \"...\"` or `table = \"...\"`",
            ))
        })?;
    }

    if table_name.is_some() && table_alias.is_some() {
        return Err(syn::Error::new(
            input.span(),
            "use either `table_name` or `table` in #[model(...)], not both",
        ));
    }

    if table_name.is_none() {
        table_name = table_alias;
    }

    Ok(table_name)
}

fn is_string_type(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == "String")
            .unwrap_or(false),
        _ => false,
    }
}

fn is_i64_type(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == "i64")
            .unwrap_or(false),
        _ => false,
    }
}

fn is_option_i64_type(ty: &Type) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };

    let Some(last) = tp.path.segments.last() else {
        return false;
    };

    if last.ident != "Option" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };

    if args.args.len() != 1 {
        return false;
    }

    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return false;
    };

    is_i64_type(inner)
}

fn map_rust_type_to_column_type(ty: &Type) -> (proc_macro2::TokenStream, bool) {
    if let Some(inner) = get_option_inner(ty) {
        let (tokens, _) = map_rust_type_to_column_type(inner);
        return (tokens, true);
    }

    if is_i64_type(ty) {
        (quote!(BigInt), false)
    } else if is_type(ty, "i32") {
        (quote!(Int), false)
    } else if is_string_type(ty) {
        (quote!(Text), false)
    } else if is_type(ty, "bool") {
        (quote!(Boolean), false)
    } else if is_type(ty, "f64") {
        (quote!(Float), false)
    } else if is_type(ty, "DateTime") {
        (quote!(DateTime), false)
    } else if is_type(ty, "Value") {
        (quote!(Json), false)
    } else if is_type(ty, "Uuid") {
        (quote!(Uuid), false)
    } else {
        (quote!(Text), false)
    }
}

fn get_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else {
        return None;
    };

    let last = tp.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };

    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
        Some(inner)
    } else {
        None
    }
}

fn is_type(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false),
        _ => false,
    }
}
