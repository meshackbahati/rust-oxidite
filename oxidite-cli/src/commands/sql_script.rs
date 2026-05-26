use std::env;
use crate::env as cli_env;

pub fn load_database_url() -> Result<String, Box<dyn std::error::Error>> {
    cli_env::load_env()?;
    cli_env::get_database_url()
}

fn normalize_database_url(url: &str) -> String {
    if let Some(path_and_query) = url.strip_prefix("sqlite://") {
        if path_and_query.starts_with('/')
            || path_and_query.starts_with("./")
            || path_and_query.starts_with("../")
            || path_and_query.is_empty()
        {
            return url.to_string();
        }

        if let Some((path, query)) = path_and_query.split_once('?') {
            return format!("sqlite://./{path}?{query}");
        }

        return format!("sqlite://./{path_and_query}");
    }

    url.to_string()
}

pub async fn execute_sql_script(
    db: &impl oxidite_db::Database,
    script: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for statement in split_sql_statements(script) {
        db.execute(&statement).await?;
    }
    Ok(())
}

pub fn split_sql_statements(script: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = script.chars().collect();

    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();

        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                if !current.ends_with(' ') {
                    current.push(' ');
                }
            }
            i += 1;
            continue;
        }

        if in_block_comment {
            if c == '*' && next == Some('/') {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if !in_single && !in_double {
            if c == '-' && next == Some('-') {
                in_line_comment = true;
                i += 2;
                continue;
            }

            if c == '/' && next == Some('*') {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }

        if c == '\'' && !in_double {
            in_single = !in_single;
            current.push(c);
            i += 1;
            continue;
        }

        if c == '"' && !in_single {
            in_double = !in_double;
            current.push(c);
            i += 1;
            continue;
        }

        if c == ';' && !in_single && !in_double {
            let stmt = current.trim();
            if !stmt.is_empty() {
                statements.push(stmt.to_string());
            }
            current.clear();
            i += 1;
            continue;
        }

        current.push(c);
        i += 1;
    }

    let stmt = current.trim();
    if !stmt.is_empty() {
        statements.push(stmt.to_string());
    }

    statements
}
