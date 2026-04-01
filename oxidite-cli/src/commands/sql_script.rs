use std::env;

pub fn load_database_url() -> Result<String, Box<dyn std::error::Error>> {
    use oxidite_config::Config;

    if let Ok(url) = env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Ok(normalize_database_url(&url));
        }
    }

    let config = Config::load()?;
    if !config.database.url.trim().is_empty() {
        return Ok(normalize_database_url(&config.database.url));
    }

    Ok("sqlite://./data.db".to_string())
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

#[cfg(test)]
mod tests {
    use super::{normalize_database_url, split_sql_statements};

    #[test]
    fn splitter_handles_semicolons_in_strings() {
        let sql = "INSERT INTO x VALUES ('a;b'); INSERT INTO x VALUES (\"c;d\");";
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn splitter_ignores_line_and_block_comments() {
        let sql = r#"
            -- before
            CREATE TABLE users(id INTEGER); /* block; comment */
            INSERT INTO users(id) VALUES (1); -- tail
        "#;
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].starts_with("CREATE TABLE users"));
        assert!(statements[1].starts_with("INSERT INTO users"));
    }

    #[test]
    fn normalizes_relative_sqlite_file_urls() {
        assert_eq!(
            normalize_database_url("sqlite://data.db"),
            "sqlite://./data.db"
        );
        assert_eq!(
            normalize_database_url("sqlite://db/app.db?mode=rwc"),
            "sqlite://./db/app.db?mode=rwc"
        );
        assert_eq!(
            normalize_database_url("sqlite:///tmp/data.db"),
            "sqlite:///tmp/data.db"
        );
        assert_eq!(
            normalize_database_url("postgres://localhost/app"),
            "postgres://localhost/app"
        );
    }
}
