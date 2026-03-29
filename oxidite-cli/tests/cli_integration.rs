use std::fs;
use std::path::Path;
use std::process::Command;

fn run_cli(current_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oxidite-cli"))
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("failed to execute oxidite-cli")
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

    let make_model = run_cli(&project, &["make", "model", "UserProfile"]);
    assert!(
        make_model.status.success(),
        "make model failed: {}",
        String::from_utf8_lossy(&make_model.stderr)
    );

    let model_file = project.join("src/models/user_profile.rs");
    assert!(model_file.exists());
    let content = fs::read_to_string(model_file).expect("read generated model");
    assert!(content.contains("#[model(table = \"user_profiles\")]") );
    assert!(content.contains("pub struct UserProfile"));

    let make_job = run_cli(&project, &["make", "job", "SendDigest"]);
    assert!(make_job.status.success());
    assert!(project.join("src/jobs/send_digest.rs").exists());

    let make_policy = run_cli(&project, &["make", "policy", "Post"]);
    assert!(make_policy.status.success());
    assert!(project.join("src/policies/post.rs").exists());

    let make_event = run_cli(&project, &["make", "event", "UserSignedUp"]);
    assert!(make_event.status.success());
    assert!(project.join("src/events/user_signed_up.rs").exists());
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
