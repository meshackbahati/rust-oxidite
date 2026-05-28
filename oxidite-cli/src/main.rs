use clap::{Parser, Subcommand};
use std::process::Command;

mod commands;
mod alias_hint;

use commands::output::{error, info, header, compile_error, runtime_error, build_failed, build_success, init_colors};

#[derive(Parser)]
#[command(name = "oxidite")]
#[command(version)]
#[command(about = "Oxidite Framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the current project in release mode
    Serve {
        /// Address to bind to
        #[arg(short, long)]
        addr: Option<String>,
        /// Host override passed to the project as SERVER_HOST
        #[arg(long)]
        host: Option<String>,
        /// Port override passed to the project as SERVER_PORT
        #[arg(long)]
        port: Option<u16>,
        /// Environment override passed to the project as OXIDITE_ENV
        #[arg(long)]
        env: Option<String>,
    },
    /// Create a new Oxidite project
    New {
        /// Project name
        name: String,
        /// Project type (api, fullstack, microservice, serverless)
        #[arg(short = 't', long = "project-type", visible_alias = "type")]
        project_type: Option<String>,
        /// Template alias for project type (api, web, fullstack, minimal, microservice, serverless)
        #[arg(long)]
        template: Option<String>,
        /// Comma-separated feature list accepted for compatibility with published docs
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,
    },
    /// Generate code
    Generate {
        #[command(subcommand)]
        generator: Generator,
    },
    /// Generate code using the legacy alias
    #[command(hide = true)]
    Make {
        #[command(subcommand)]
        generator: Generator,
    },
    /// Database migrations
    Migrate {
        #[command(subcommand)]
        migration: Option<MigrateCommand>,
    },
    /// Generate a declarative migration by diffing models with current schema
    #[command(name = "make-migrations")]
    MakeMigrations {
        /// Name of the migration
        name: Option<String>,
        /// Dry run: show SQL without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Roll back the last migration using the documented alias
    #[command(name = "migrate:rollback", hide = true)]
    MigrateRollback,
    /// Database seeders
    Seed {
        #[command(subcommand)]
        seeder: Option<SeedCommand>,
    },
    /// Run seeders using the documented alias
    #[command(name = "db:seed", hide = true)]
    DbSeed,
    /// Queue management
    Queue {
        #[command(subcommand)]
        queue: QueueCommand,
    },
    /// Start queue worker using the documented alias
    #[command(name = "queue:work", hide = true)]
    QueueWork {
        #[arg(short, long, default_value_t = 4)]
        workers: usize,
    },
    /// List queue statistics using the documented alias
    #[command(name = "queue:list", hide = true)]
    QueueList,
    /// List dead letter queue using the documented alias
    #[command(name = "queue:dlq", hide = true)]
    QueueDlq,
    /// Clear pending jobs using the documented alias
    #[command(name = "queue:clear", hide = true)]
    QueueClear,
    /// System health check
    Doctor,
    /// Production build
    Build {
        #[arg(short, long)]
        release: bool,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        features: Option<String>,
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Start development server with hot reload
    Dev {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long = "watch")]
        watch: Vec<String>,
        #[arg(long = "ignore")]
        ignore: Vec<String>,
        #[arg(long = "hot-reload")]
        hot_reload: bool,
        #[arg(long = "no-hot-reload", conflicts_with = "hot_reload")]
        no_hot_reload: bool,
    },
    /// Print the installed CLI version
    Version,
    /// Open an interactive console (REPL) for your project
    Tinker,
    /// Benchmark/profile an HTTP target URL
    Profile {
        /// Target URL to benchmark
        url: String,
        /// Concurrency level (number of parallel workers)
        #[arg(short, long, default_value_t = 10)]
        concurrency: usize,
        /// Total number of requests to perform
        #[arg(short, long, default_value_t = 100)]
        requests: usize,
    },
    /// Run a single Rust file as a script (standalone or within a project)
    Run {
        /// Path to the Rust file to execute
        file: String,
        /// Extra dependencies to include (crate names, comma-separated)
        #[arg(long)]
        deps: Option<String>,
    },
    /// Process management (PM2-style)
    #[command(name = "pm2")]
    Process {
        #[command(subcommand)]
        action: ProcessAction,
    },
}

#[derive(Subcommand)]
enum ProcessAction {
    /// Start a process in the background
    Start {
        /// Process name
        name: Option<String>,
        /// Run in release mode
        #[arg(long)]
        release: bool,
    },
    /// Stop a running process
    Stop {
        /// Process name or ID
        identifier: Option<String>,
    },
    /// Restart a process
    Restart {
        /// Process name or ID
        identifier: Option<String>,
    },
    /// List all running processes
    List,
    /// Show detailed info about a process
    Info {
        /// Process name or ID
        identifier: String,
    },
    /// Monitor processes in real-time
    Monitor,
}

#[derive(Subcommand)]
enum Generator {
    /// Generate a model
    Model {
        name: String,
        #[arg(value_name = "FIELD")]
        fields: Vec<String>,
    },
    /// Generate a route module
    Route { name: String },
    /// Generate a controller
    Controller { name: String },
    /// Generate middleware
    Middleware { name: String },
    /// Generate a service
    Service { name: String },
    /// Generate a validator
    Validator { name: String },
    /// Generate a background job
    Job { name: String },
    /// Generate an authorization policy
    Policy { name: String },
    /// Generate a domain event
    Event { name: String },
    /// Generate a migration file
    Migration { name: String },
    /// Generate a seeder file
    Seeder { name: String },
}

#[derive(Subcommand)]
enum MigrateCommand {
    /// Create a new migration
    Create { name: String },
    /// Run pending migrations
    Run,
    /// Revert the last migration
    Revert,
    /// Show migration status
    Status,
    /// Generate a declarative migration
    #[command(name = "make")]
    Make {
        /// Name of the migration
        name: Option<String>,
        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SeedCommand {
    /// Run database seeders
    Run,
    /// Create a new seeder
    Create { name: String },
}

#[derive(Subcommand)]
enum QueueCommand {
    /// Start queue worker
    Work {
        #[arg(short, long, default_value_t = 4)]
        workers: usize,
    },
    /// List queue statistics
    List,
    /// List dead letter queue
    Dlq,
    /// Clear all pending jobs
    Clear,
}

#[tokio::main]
async fn main() {
    init_colors();
    alias_hint::print_alias_hint();
    
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Serve {
            addr,
            host,
            port,
            env,
        } => {
            match resolve_run_options(addr, host, port, env) {
                Ok(options) => commands::dev::run_project_once(true, &options).map_err(|e| e as Box<dyn std::error::Error>),
                Err(e) => Err(e),
            }
        }
        Commands::New {
            name,
            project_type,
            template,
            features,
        } => {
            commands::create_project(&name, project_type, template, &features)
        }
        Commands::Make { generator } | Commands::Generate { generator } => {
            run_generator(generator)
        }
        Commands::Migrate { migration } => {
            match migration.unwrap_or(MigrateCommand::Run) {
                MigrateCommand::Create { name } => commands::migrate::create_migration(&name),
                MigrateCommand::Run => commands::migrate::run_migrations().await,
                MigrateCommand::Revert => commands::migrate::revert_migration().await,
                MigrateCommand::Status => commands::migrate::migration_status().await,
                MigrateCommand::Make { name, dry_run } => commands::migrate::declarative::make_migrations(name, dry_run).await,
            }
        }
        Commands::MakeMigrations { name, dry_run } => {
            commands::migrate::declarative::make_migrations(name, dry_run).await
        }
        Commands::MigrateRollback => {
            commands::migrate::revert_migration().await
        }
        Commands::Seed { seeder } => {
            match seeder.unwrap_or(SeedCommand::Run) {
                SeedCommand::Run => commands::seed::run_seeders().await,
                SeedCommand::Create { name } => commands::seed::create_seeder(&name),
            }
        }
        Commands::DbSeed => {
            commands::seed::run_seeders().await
        }
        Commands::Queue { queue } => {
            run_queue_command(queue).await
        }
        Commands::QueueWork { workers } => {
            commands::queue::queue_work(workers).await
        }
        Commands::QueueList => {
            commands::queue::queue_list().await
        }
        Commands::QueueDlq => {
            commands::queue::queue_dlq().await
        }
        Commands::QueueClear => {
            commands::queue::queue_clear().await
        }
        Commands::Doctor => {
            commands::doctor::run_doctor()
        }
        Commands::Build {
            release,
            profile,
            target,
            features,
            verbose,
        } => {
            build_project(release, profile, target, features, verbose)
        }
        Commands::Dev {
            host,
            port,
            env,
            watch,
            ignore,
            hot_reload: _,
            no_hot_reload,
        } => {
            let options = commands::dev::DevOptions {
                run: commands::dev::RunOptions { host, port, env },
                watch: watch.into_iter().map(Into::into).collect(),
                ignore,
                hot_reload: !no_hot_reload,
            };
            commands::dev::start_dev_server(options)
        }
        Commands::Version => {
            println!("oxidite {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Tinker => {
            commands::tinker::run_tinker()
        }
        Commands::Profile { url, concurrency, requests } => {
            commands::profile::run(&url, concurrency, requests).await;
            Ok(())
        }
        Commands::Run { file, deps } => {
            commands::run::run_file(&file, deps.as_deref())
        }
        Commands::Process { action } => {
            match action {
                ProcessAction::Start { name, release } => {
                    commands::process_manager::start_process(name, release)
                }
                ProcessAction::Stop { identifier } => {
                    commands::process_manager::stop_process(identifier)
                }
                ProcessAction::Restart { identifier } => {
                    commands::process_manager::restart_process(identifier)
                }
                ProcessAction::List => {
                    commands::process_manager::list_processes()
                }
                ProcessAction::Info { identifier } => {
                    commands::process_manager::show_process(&identifier)
                }
                ProcessAction::Monitor => {
                    commands::process_manager::monitor_processes()
                }
            }
        }
    };

    // Handle errors with proper formatting
    if let Err(err) = result {
        let err_str = err.to_string();
        
        // Categorize error by type and display with appropriate formatting
        if err_str.contains("not found") || err_str.contains("No such file") || err_str.contains("does not exist") {
            compile_error(&err_str);
        } else if err_str.contains("build") || err_str.contains("compil") || err_str.contains("syntax") {
            compile_error(&err_str);
        } else if err_str.contains("connection") || err_str.contains("database") || err_str.contains("timeout") {
            runtime_error(&err_str);
        } else if err_str.contains("permission") || err_str.contains("denied") {
            error(&format!("Permission denied: {}", err_str));
        } else {
            error(&err_str);
        }
        
        std::process::exit(1);
    }
}

fn run_generator(generator: Generator) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match generator {
        Generator::Model { name, fields } => commands::make::make_model(&name, &fields)?,
        Generator::Route { name } => commands::make::make_route(&name)?,
        Generator::Controller { name } => commands::make::make_controller(&name)?,
        Generator::Middleware { name } => commands::make::make_middleware(&name)?,
        Generator::Service { name } => commands::make::make_service(&name)?,
        Generator::Validator { name } => commands::make::make_validator(&name)?,
        Generator::Job { name } => commands::make::make_job(&name)?,
        Generator::Policy { name } => commands::make::make_policy(&name)?,
        Generator::Event { name } => commands::make::make_event(&name)?,
        Generator::Migration { name } => commands::migrate::create_migration(&name)?,
        Generator::Seeder { name } => commands::seed::create_seeder(&name)?,
    }
    Ok(())
}

async fn run_queue_command(
    queue: QueueCommand,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match queue {
        QueueCommand::Work { workers } => commands::queue::queue_work(workers).await?,
        QueueCommand::List => commands::queue::queue_list().await?,
        QueueCommand::Dlq => commands::queue::queue_dlq().await?,
        QueueCommand::Clear => commands::queue::queue_clear().await?,
    }
    Ok(())
}

fn build_project(
    release: bool,
    profile: Option<String>,
    target: Option<String>,
    features: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    header("Building Oxidite project");

    let mut command = Command::new("cargo");
    command.arg("build");

    if let Some(profile) = profile {
        command.arg("--profile").arg(profile);
    } else if release {
        command.arg("--release");
        info("Building in release mode");
    }

    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    if let Some(features) = features {
        command.arg("--features").arg(features);
    }

    if verbose {
        command.arg("-v");
    }

    let status = command
        .status()
        .map_err(|err| Box::new(std::io::Error::new(std::io::ErrorKind::Other, err.to_string())) as Box<dyn std::error::Error>)?;

    if status.success() {
        build_success("Compilation completed successfully");
        Ok(())
    } else {
        build_failed("Cargo build process returned errors");
        Err("Build failed".into())
    }
}

fn resolve_run_options(
    addr: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    env: Option<String>,
) -> Result<commands::dev::RunOptions, Box<dyn std::error::Error>> {
    match parse_addr_opt(&addr) {
        Ok((addr_host, addr_port)) => {
            Ok(commands::dev::RunOptions {
                host: host.or(addr_host),
                port: port.or(addr_port),
                env,
            })
        }
        Err(e) => Err(e),
    }
}

fn parse_addr_opt(addr: &Option<String>) -> Result<(Option<String>, Option<u16>), Box<dyn std::error::Error>> {
    if addr.is_none() {
        return Ok((None, None));
    }
    
    let addr = addr.as_ref().unwrap();
    let Some((host, port)) = addr.rsplit_once(':') else {
        return Err(format!("invalid address `{addr}`; expected host:port").into());
    };

    let port = port
        .parse::<u16>()
        .map_err(|_| format!("invalid port in address `{addr}`"))?;

    Ok((Some(host.to_string()), Some(port)))
}

#[cfg(test)]
mod tests {
    use super::parse_addr_opt;

    #[test]
    fn parses_host_and_port_from_addr() {
        let (host, port) = parse_addr_opt(&Some("127.0.0.1:8080".to_string())).unwrap();
        assert_eq!(host.as_deref(), Some("127.0.0.1"));
        assert_eq!(port, Some(8080));
    }

    #[test]
    fn rejects_invalid_addr() {
        assert!(parse_addr_opt(&Some("not-an-addr".to_string())).is_err());
    }
}
