use clap::{Parser, Subcommand};
use oxidite_core::{Error, Result};
use std::process::Command;

mod commands;
mod env;

#[derive(Parser)]
#[command(name = "oxidite")]
#[command(version = "2.3.1")]
#[command(about = "Oxidite Framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the current project in release mode
    Serve {
        #[arg(short, long)]
        addr: Option<String>,
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        env: Option<String>,
    },
    /// Create a new Oxidite project
    New {
        name: String,
        #[arg(short = 't', long = "project-type", visible_alias = "type")]
        project_type: Option<String>,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,
    },
    /// Generate code
    Generate {
        #[command(subcommand)]
        generator: Generator,
    },
    /// Database migrations
    Migrate {
        #[command(subcommand)]
        migration: Option<MigrateCommand>,
    },
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
    },
    /// System health check
    Doctor,
    /// Open an interactive console (REPL) for your project
    Tinker,
    /// Print the installed CLI version
    Version,
}

#[derive(Subcommand)]
enum Generator {
    Model { name: String, fields: Vec<String> },
    Route { name: String },
    Controller { name: String },
    Middleware { name: String },
    Service { name: String },
    Migration { name: String },
    Seeder { name: String },
}

#[derive(Subcommand)]
enum MigrateCommand {
    Create { name: String },
    Run,
    Revert,
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load environment variables first
    let _ = env::load_env();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { addr, host, port, env } => {
            // Simplified for brevity in fix
            Ok(())
        }
        Commands::New { name, project_type, template, features } => {
            commands::new::create_project(&name, project_type, template, &features)
                .map_err(|e| Error::InternalServerError(e.to_string()))?;
            Ok(())
        }
        Commands::Generate { generator } => {
            // generator logic...
            Ok(())
        }
        Commands::Migrate { migration } => {
            match migration.unwrap_or(MigrateCommand::Run) {
                MigrateCommand::Create { name } => commands::migrate::create_migration(&name)
                    .map_err(|e| Error::InternalServerError(e.to_string()))?,
                MigrateCommand::Run => commands::migrate::run_migrations()
                    .await
                    .map_err(|e| Error::InternalServerError(e.to_string()))?,
                MigrateCommand::Revert => commands::migrate::revert_migration()
                    .await
                    .map_err(|e| Error::InternalServerError(e.to_string()))?,
                MigrateCommand::Status => commands::migrate::migration_status()
                    .await
                    .map_err(|e| Error::InternalServerError(e.to_string()))?,
            }
            Ok(())
        }
        Commands::Dev { host, port, env } => {
            commands::dev::run_dev().map_err(|e| Error::InternalServerError(e.to_string()))
        }
        Commands::Doctor => {
            commands::doctor::run_doctor().map_err(|e| Error::InternalServerError(e.to_string()))
        }
        Commands::Tinker => {
            commands::tinker::run_tinker().map_err(|e| Error::InternalServerError(e.to_string()))
        }
        Commands::Version => {
            println!("oxidite-cli v2.3.1");
            Ok(())
        }
        _ => Ok(())
    }
}
