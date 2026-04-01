use clap::{Parser, Subcommand};
use oxidite_core::{Error, Result};
use std::process::Command;

mod commands;

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
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            addr,
            host,
            port,
            env,
        } => {
            let options = resolve_run_options(addr, host, port, env)?;
            commands::dev::run_project_once(true, &options)
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::New {
            name,
            project_type,
            template,
            features,
        } => {
            commands::create_project(&name, project_type, template, &features)
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Make { generator } | Commands::Generate { generator } => {
            run_generator(generator).map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Migrate { migration } => {
            match migration.unwrap_or(MigrateCommand::Run) {
                MigrateCommand::Create { name } => commands::migrate::create_migration(&name)
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
                MigrateCommand::Run => commands::migrate::run_migrations()
                    .await
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
                MigrateCommand::Revert => commands::migrate::revert_migration()
                    .await
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
                MigrateCommand::Status => commands::migrate::migration_status()
                    .await
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
            }
            Ok(())
        }
        Commands::MigrateRollback => {
            commands::migrate::revert_migration()
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Seed { seeder } => {
            match seeder.unwrap_or(SeedCommand::Run) {
                SeedCommand::Run => commands::seed::run_seeders()
                    .await
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
                SeedCommand::Create { name } => commands::seed::create_seeder(&name)
                    .map_err(|err| Error::InternalServerError(err.to_string()))?,
            }
            Ok(())
        }
        Commands::DbSeed => {
            commands::seed::run_seeders()
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Queue { queue } => {
            run_queue_command(queue)
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::QueueWork { workers } => {
            commands::queue::queue_work(workers)
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::QueueList => {
            commands::queue::queue_list()
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::QueueDlq => {
            commands::queue::queue_dlq()
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::QueueClear => {
            commands::queue::queue_clear()
                .await
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Doctor => {
            commands::doctor::run_doctor()
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Build {
            release,
            profile,
            target,
            features,
            verbose,
        } => {
            build_project(release, profile, target, features, verbose)?;
            Ok(())
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
                .map_err(|err| Error::InternalServerError(err.to_string()))?;
            Ok(())
        }
        Commands::Version => {
            println!("oxidite {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
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
) -> Result<()> {
    println!("🔨 Building Oxidite project...");

    let mut command = Command::new("cargo");
    command.arg("build");

    if let Some(profile) = profile {
        command.arg("--profile").arg(profile);
    } else if release {
        command.arg("--release");
        println!("📦 Building in release mode");
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
        .map_err(|err| Error::InternalServerError(err.to_string()))?;

    if status.success() {
        println!("✅ Build completed successfully");
        Ok(())
    } else {
        Err(Error::InternalServerError("Build failed".to_string()))
    }
}

fn resolve_run_options(
    addr: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    env: Option<String>,
) -> Result<commands::dev::RunOptions> {
    let (addr_host, addr_port) = if let Some(addr) = addr {
        parse_addr(&addr)?
    } else {
        (None, None)
    };

    Ok(commands::dev::RunOptions {
        host: host.or(addr_host),
        port: port.or(addr_port),
        env,
    })
}

fn parse_addr(addr: &str) -> Result<(Option<String>, Option<u16>)> {
    let Some((host, port)) = addr.rsplit_once(':') else {
        return Err(Error::InternalServerError(format!(
            "invalid address `{addr}`; expected host:port"
        )));
    };

    let port = port
        .parse::<u16>()
        .map_err(|_| Error::InternalServerError(format!("invalid port in address `{addr}`")))?;

    Ok((Some(host.to_string()), Some(port)))
}

#[cfg(test)]
mod tests {
    use super::parse_addr;

    #[test]
    fn parses_host_and_port_from_addr() {
        let (host, port) = parse_addr("127.0.0.1:8080").unwrap();
        assert_eq!(host.as_deref(), Some("127.0.0.1"));
        assert_eq!(port, Some(8080));
    }

    #[test]
    fn rejects_invalid_addr() {
        assert!(parse_addr("not-an-addr").is_err());
    }
}
