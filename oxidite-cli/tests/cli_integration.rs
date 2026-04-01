use std::fs;
use std::path::Path;
use std::process::Command;

fn run_cli(current_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oxidite"))
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("failed to execute oxidite")
}

#[test]
fn new_project_and_make_model_workflow() {
    let temp = tempfile::tempdir().expect("temp dir");

    let create = run_cli(temp.path(), &["new", "demo", "--project-type", "api"]);
    assert!(
        create.status.success(),
        "new command failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let project = temp.path().join("demo");
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("src/main.rs").exists());
    assert!(project.join("migrations").exists());
    assert!(project.join("seeds").exists());

    let make_model = run_cli(&project, &["generate", "model", "UserProfile"]);
    assert!(
        make_model.status.success(),
        "generate model failed: {}",
        String::from_utf8_lossy(&make_model.stderr)
    );

    let model_file = project.join("src/models/user_profile.rs");
    assert!(model_file.exists());
    let content = fs::read_to_string(model_file).expect("read generated model");
    assert!(content.contains("#[model(table = \"user_profiles\")]"));
    assert!(content.contains("pub struct UserProfile"));
    assert!(content.contains("use oxidite::db::{Model, sqlx};"));
    assert!(fs::read_to_string(project.join("src/models/mod.rs"))
        .expect("read models mod")
        .contains("pub mod user_profile;"));

    let make_job = run_cli(&project, &["generate", "job", "SendDigest"]);
    assert!(make_job.status.success());
    assert!(project.join("src/jobs/send_digest.rs").exists());

    let make_route = run_cli(&project, &["generate", "route", "users"]);
    assert!(make_route.status.success());
    assert!(project.join("src/routes/users.rs").exists());
    let routes_mod =
        fs::read_to_string(project.join("src/routes/mod.rs")).expect("read routes mod");
    assert!(routes_mod.contains("pub mod users;"));
    assert!(routes_mod.contains("register_generated(router);"));
    assert!(routes_mod.contains("users::register(router);"));

    let make_service = run_cli(&project, &["generate", "service", "Billing"]);
    assert!(make_service.status.success());
    assert!(project.join("src/services/billing.rs").exists());
    assert!(fs::read_to_string(project.join("src/services/mod.rs"))
        .expect("read services mod")
        .contains("pub mod billing;"));

    let make_validator = run_cli(&project, &["generate", "validator", "CreateUser"]);
    assert!(make_validator.status.success());
    assert!(project.join("src/validators/create_user.rs").exists());

    let make_policy = run_cli(&project, &["make", "policy", "Post"]);
    assert!(make_policy.status.success());
    assert!(project.join("src/policies/post.rs").exists());

    let make_event = run_cli(&project, &["make", "event", "UserSignedUp"]);
    assert!(make_event.status.success());
    assert!(project.join("src/events/user_signed_up.rs").exists());
}

#[test]
fn generator_repairs_legacy_module_layout() {
    let temp = tempfile::tempdir().expect("temp dir");

    let create = run_cli(
        temp.path(),
        &["new", "legacy_demo", "--project-type", "api"],
    );
    assert!(create.status.success());

    let project = temp.path().join("legacy_demo");

    fs::write(
        project.join("src/main.rs"),
        r#"use oxidite::prelude::*;

mod routes;
mod controllers;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let mut router = Router::new();
    routes::register(&mut router);

    let server = Server::new(router);
    println!("Server running on http://{}", addr);
    server.listen(addr.parse().unwrap()).await
}
"#,
    )
    .expect("write legacy main");

    fs::write(
        project.join("src/routes/mod.rs"),
        r#"use oxidite::prelude::*;

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
}

async fn health(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({"status": "ok"})))
}
"#,
    )
    .expect("write legacy routes mod");

    fs::write(project.join("src/services/mod.rs"), "").expect("reset services mod");

    let generate_service = run_cli(&project, &["generate", "service", "Billing"]);
    assert!(
        generate_service.status.success(),
        "generate service failed: {}",
        String::from_utf8_lossy(&generate_service.stderr)
    );

    let main_rs = fs::read_to_string(project.join("src/main.rs")).expect("read main");
    assert!(main_rs.contains("mod services;"));

    let services_mod =
        fs::read_to_string(project.join("src/services/mod.rs")).expect("read services mod");
    assert!(services_mod.contains("pub mod billing;"));

    let generate_route = run_cli(&project, &["generate", "route", "users"]);
    assert!(
        generate_route.status.success(),
        "generate route failed: {}",
        String::from_utf8_lossy(&generate_route.stderr)
    );

    let routes_mod =
        fs::read_to_string(project.join("src/routes/mod.rs")).expect("read routes mod");
    assert!(routes_mod.contains("pub mod users;"));
    assert!(routes_mod.contains("register_generated(router);"));
    assert!(routes_mod.contains("users::register(router);"));
}

#[test]
fn generator_exports_modules_from_lib_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project = temp.path();

    fs::create_dir(project.join("src")).expect("create src");
    fs::write(
        project.join("src/lib.rs"),
        r#"pub mod routes;

pub fn crate_name() -> &'static str {
    "demo"
}
"#,
    )
    .expect("write lib.rs");
    fs::write(project.join("src/routes.rs"), "pub fn register() {}\n").expect("write routes.rs");

    let generate_service = run_cli(project, &["generate", "service", "Billing"]);
    assert!(
        generate_service.status.success(),
        "generate service failed: {}",
        String::from_utf8_lossy(&generate_service.stderr)
    );

    let lib_rs = fs::read_to_string(project.join("src/lib.rs")).expect("read lib.rs");
    assert!(lib_rs.contains("pub mod services;"));

    let services_mod =
        fs::read_to_string(project.join("src/services/mod.rs")).expect("read services mod");
    assert!(services_mod.contains("pub mod billing;"));
}

#[test]
fn documented_aliases_work() {
    let temp = tempfile::tempdir().expect("temp dir");

    let create = run_cli(temp.path(), &["new", "docs_demo", "--template", "api"]);
    assert!(
        create.status.success(),
        "new --template failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let project = temp.path().join("docs_demo");

    let generate_model = run_cli(
        &project,
        &["generate", "model", "User", "email:string", "age:integer"],
    );
    assert!(
        generate_model.status.success(),
        "generate model failed: {}",
        String::from_utf8_lossy(&generate_model.stderr)
    );

    let model_file = project.join("src/models/user.rs");
    let content = fs::read_to_string(model_file).expect("read generated model");
    assert!(content.contains("pub email: String"));
    assert!(content.contains("pub age: i64"));

    let migration = run_cli(&project, &["generate", "migration", "create_users_table"]);
    assert!(
        migration.status.success(),
        "generate migration failed: {}",
        String::from_utf8_lossy(&migration.stderr)
    );

    let migration_path = fs::read_dir(project.join("migrations"))
        .expect("read migrations dir")
        .map(|entry| entry.expect("dir entry").path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .expect("find migration file");

    fs::write(
        &migration_path,
        "-- migrate:up\nCREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL);\n\n-- migrate:down\nDROP TABLE users;\n",
    )
    .expect("write migration");

    let migrate = run_cli(&project, &["migrate"]);
    assert!(
        migrate.status.success(),
        "bare migrate failed: {}",
        String::from_utf8_lossy(&migrate.stderr)
    );
}

#[test]
fn migration_and_seeder_scaffolding_work() {
    let temp = tempfile::tempdir().expect("temp dir");

    let create = run_cli(temp.path(), &["new", "demo2", "--project-type", "api"]);
    assert!(create.status.success());

    let project = temp.path().join("demo2");

    let migration = run_cli(&project, &["migrate", "create", "create_users"]);
    assert!(
        migration.status.success(),
        "migrate create failed: {}",
        String::from_utf8_lossy(&migration.stderr)
    );

    let migrations_dir = project.join("migrations");
    let mut found_migration = false;
    for entry in fs::read_dir(&migrations_dir).expect("read migrations dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let content = fs::read_to_string(path).expect("read migration file");
            assert!(content.contains("-- migrate:up"));
            assert!(content.contains("-- migrate:down"));
            found_migration = true;
            break;
        }
    }
    assert!(found_migration, "no migration sql file was created");

    let seeder = run_cli(&project, &["seed", "create", "users_seed"]);
    assert!(
        seeder.status.success(),
        "seed create failed: {}",
        String::from_utf8_lossy(&seeder.stderr)
    );

    let seeds_dir = project.join("seeds");
    let mut found_seed = false;
    for entry in fs::read_dir(&seeds_dir).expect("read seeds dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let content = fs::read_to_string(path).expect("read seed file");
            assert!(content.contains("INSERT INTO users"));
            found_seed = true;
            break;
        }
    }
    assert!(found_seed, "no seed sql file was created");
}
