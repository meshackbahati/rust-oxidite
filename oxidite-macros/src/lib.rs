use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitStr, Type,
};

#[proc_macro_derive(Model, attributes(validate, model))]
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

    if let Some(id_field) = find_field("id") {
        if !is_i64_type(&id_field.ty) {
            return Err(syn::Error::new(
                id_field.ty.span(),
                "Model derive requires `id` to be of type i64",
            ));
        }
    }

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

    let mut validation_checks = Vec::new();
    for field in &named_fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.span(), "Expected named field"))?;

        for attr in &field.attrs {
            if attr.path().is_ident("validate") {
                let attr_str = attr.to_token_stream().to_string();
                if attr_str.contains("email") {
                    if !is_string_type(&field.ty) {
                        return Err(syn::Error::new(
                            field.ty.span(),
                            "#[validate(email)] can only be used on String fields",
                        ));
                    }

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

            fn is_persisted(&self) -> bool {
                self.id > 0
            }
        }
    };

    Ok(expanded)
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
