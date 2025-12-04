use clap::{Parser, Subcommand};
use oxidite_core::{Router, Server, OxiditeRequest, OxiditeResponse, Result};
use oxidite_middleware::{ServiceBuilder, LoggerLayer};
use http_body_util::Full;
use bytes::Bytes;
mod commands;

#[derive(Parser)]
#[command(name = "oxidite")]
#[command(about = "Oxidite Framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        addr: String,
    },
    /// Create a new Oxidite project
    New {
        /// Project name
        name: String,
    },
    /// Generate code
    Make {
        #[command(subcommand)]
        generator: Generator,
    },
    /// Database migrations
    Migrate {
        #[command(subcommand)]
        migration: MigrateCommand,
    },
}

#[derive(Subcommand)]
enum Generator {
    /// Generate a model
    Model { name: String },
    /// Generate a controller
    Controller { name: String },
    /// Generate middleware
    Middleware { name: String },
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

async fn hello(_req: OxiditeRequest) -> Result<OxiditeResponse> {
    Ok(hyper::Response::new(Full::new(Bytes::from("Hello, Oxidite!"))))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { addr } => {
            let mut router = Router::new();
            router.get("/", hello);

            let service = ServiceBuilder::new()
                .layer(LoggerLayer)
                .service(router);

            let server = Server::new(service);
            println!("🚀 Server running on http://{}", addr);
            server.listen(addr.parse().unwrap()).await
        }
        Commands::New { name } => {
            commands::create_project(&name)
                .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
            Ok(())
        }
        Commands::Make { generator } => {
            match generator {
                Generator::Model { name } => {
                    commands::make::make_model(&name)
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
                Generator::Controller { name } => {
                    commands::make::make_controller(&name)
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
                Generator::Middleware { name } => {
                    commands::make::make_middleware(&name)
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
            }
            Ok(())
        }
        Commands::Migrate { migration } => {
            match migration {
                MigrateCommand::Create { name } => {
                    commands::migrate::create_migration(&name)
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
                MigrateCommand::Run => {
                    commands::migrate::run_migrations()
                        .await
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
                MigrateCommand::Revert => {
                    commands::migrate::revert_migration()
                        .await
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
                MigrateCommand::Status => {
                    commands::migrate::migration_status()
                        .await
                        .map_err(|e| oxidite_core::Error::Server(e.to_string()))?;
                }
            }
            Ok(())
        }
    }
}
