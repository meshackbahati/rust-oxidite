use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

// Embed the Oxidite SVG logo at compile time so `oxidite new` can write it
// to the generated project's `public/images/` directory.
const OXIDITE_LOGO_SVG: &str = include_str!("../templates/oxidite.svg");

#[derive(Debug, Clone, Copy)]
pub enum ProjectType {
    Fullstack,
    Api,
    Microservice,
    Serverless,
}

impl ProjectType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fullstack" => Some(Self::Fullstack),
            "web" => Some(Self::Fullstack),
            "api" => Some(Self::Api),
            "minimal" => Some(Self::Api),
            "microservice" => Some(Self::Microservice),
            "serverless" => Some(Self::Serverless),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Fullstack => "Fullstack Application",
            Self::Api => "REST API",
            Self::Microservice => "Microservice",
            Self::Serverless => "Serverless Function",
        }
    }
}

pub fn create_project(
    name: &str,
    project_type: Option<String>,
    template: Option<String>,
    requested_features: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        format!("Initializing new Oxidite project: {}", name)
            .green()
            .bold()
    );

    let explicit_type = project_type
        .as_deref()
        .or(template.as_deref())
        .map(|value| {
            ProjectType::from_str(value).ok_or(
                "Invalid project type/template. Options: fullstack, web, api, minimal, microservice, serverless",
            )
        })
        .transpose()?;

    let p_type = if let Some(p_type) = explicit_type {
        p_type
    } else {
        let selections = &[
            "Fullstack Application (Frontend + Backend)",
            "REST API (Backend only)",
            "Microservice (Minimal, specialized)",
            "Serverless Function (Event-driven)",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select project type")
            .default(0)
            .items(selections)
            .interact()?;

        match selection {
            0 => ProjectType::Fullstack,
            1 => ProjectType::Api,
            2 => ProjectType::Microservice,
            3 => ProjectType::Serverless,
            _ => unreachable!(),
        }
    };

    println!("Creating {}...", p_type.as_str().cyan());

    // Create the project directory structure.
    fs::create_dir(name)?;
    let project_path = Path::new(name);
    let src_path = project_path.join("src");

    match p_type {
        ProjectType::Serverless => {
            // Serverless-native structure: handlers are isolated functions,
            // infra-as-code lives alongside source, no MVC boilerplate.
            fs::create_dir(&src_path)?;
            fs::create_dir(src_path.join("handlers"))?;
            fs::create_dir(src_path.join("core"))?;
            fs::create_dir(src_path.join("config"))?;
            fs::create_dir(project_path.join("tests"))?;
            fs::create_dir(project_path.join("infra"))?;
            fs::create_dir(project_path.join("infra").join("aws"))?;
            fs::create_dir(project_path.join("infra").join("cloudflare"))?;
            fs::create_dir(project_path.join("config"))?;
        }
        ProjectType::Microservice => {
            // Microservice: focused scope, event-driven, independent deployability.
            // No controllers/services/middleware/validators/policies — those are
            // REST API monolith patterns. Instead: handlers for HTTP, events for
            // async messaging, queues for consumers, core for domain logic.
            fs::create_dir(&src_path)?;
            fs::create_dir(src_path.join("handlers"))?;
            fs::create_dir(src_path.join("core"))?;
            fs::create_dir(src_path.join("events"))?;
            fs::create_dir(src_path.join("queues"))?;
            fs::create_dir(src_path.join("models"))?;
            fs::create_dir(src_path.join("config"))?;
            fs::create_dir(src_path.join("utils"))?;
            fs::create_dir(project_path.join("tests"))?;
            fs::create_dir(project_path.join("migrations"))?;
            fs::create_dir(project_path.join("seeds"))?;
        }
        _ => {
            // Fullstack / API: standard MVC-style structure.
            fs::create_dir(&src_path)?;
            fs::create_dir(src_path.join("models"))?;
            fs::create_dir(src_path.join("routes"))?;
            fs::create_dir(src_path.join("controllers"))?;
            fs::create_dir(src_path.join("services"))?;
            fs::create_dir(src_path.join("middleware"))?;
            fs::create_dir(src_path.join("validators"))?;
            fs::create_dir(src_path.join("jobs"))?;
            fs::create_dir(src_path.join("policies"))?;
            fs::create_dir(src_path.join("events"))?;
            fs::create_dir(src_path.join("utils"))?;
            fs::create_dir(src_path.join("config"))?;
            fs::create_dir(project_path.join("tests"))?;
            fs::create_dir(project_path.join("migrations"))?;
            fs::create_dir(project_path.join("seeds"))?;

            if matches!(p_type, ProjectType::Fullstack) {
                fs::create_dir(project_path.join("templates"))?;
                fs::create_dir(project_path.join("public"))?;
                fs::create_dir(project_path.join("public/css"))?;
                fs::create_dir(project_path.join("public/js"))?;
                fs::create_dir(project_path.join("public/images"))?;
            }
        }
    }

    create_test_file(project_path, p_type)?;

    create_cargo_toml(project_path, name, p_type)?;
    create_config_toml(project_path, p_type)?;
    create_main_rs(project_path, p_type)?;
    create_boilerplate(project_path, name, p_type)?;
    create_deployment_doc(project_path, name, p_type)?;
    create_readme(project_path, name, p_type)?;

    // .gitignore — keep secrets and build artefacts out of version control.
    let gitignore = r#"/target
Cargo.lock
*.db
*.log
.env
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    if !requested_features.is_empty() {
        println!(
            "{} {}",
            "Requested feature flags recorded but not scaffolded automatically:".yellow(),
            requested_features.join(", ")
        );
    }

    println!("\n{}", "Project created successfully!".green().bold());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  oxidite migrate");
    println!("  oxidite dev");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test files (scaffolded for all project types)
// ---------------------------------------------------------------------------

fn create_test_file(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let test_content = match p_type {
        ProjectType::Serverless => r#"use oxidite_testing::{TestRequest, TestResponse, test_router};
use oxidite::prelude::*;

#[tokio::test]
async fn test_health_endpoint() {
    let mut router = Router::new();
    router.get("/", |_req: Request| async {
        Ok(Response::json_val(json!({"status": "ok"})))
    });

    let mut server = test_router(router);
    let req = TestRequest::get("/").build_oxidite();
    let resp = server.call(req).await.unwrap();

    let test_resp = TestResponse::from_oxidite_response(resp).await;
    assert!(test_resp.is_success());
}
"#,
        ProjectType::Microservice => r#"use oxidite_testing::{TestRequest, TestResponse, test_router};
use oxidite::prelude::*;

#[tokio::test]
async fn test_health_check() {
    let mut router = Router::new();
    router.get("/health", |_req: Request| async {
        Ok(Response::json_val(json!({"status": "ok"})))
    });

    let mut server = test_router(router);
    let req = TestRequest::get("/health").build_oxidite();
    let resp = server.call(req).await.unwrap();
    let test_resp = TestResponse::from_oxidite_response(resp).await;
    assert!(test_resp.is_success());
}

#[tokio::test]
async fn test_status_endpoint() {
    let mut router = Router::new();
    router.get("/api/v1/status", |_req: Request| async {
        Ok(Response::json_val(json!({
            "service": "microservice",
            "status": "ok"
        })))
    });

    let mut server = test_router(router);
    let req = TestRequest::get("/api/v1/status").build_oxidite();
    let resp = server.call(req).await.unwrap();
    let test_resp = TestResponse::from_oxidite_response(resp).await;
    assert!(test_resp.is_success());
}
"#,
        _ => r#"use oxidite_testing::{TestRequest, TestResponse, test_router};
use oxidite::prelude::*;

#[tokio::test]
async fn test_health_endpoint() {
    let mut router = Router::new();
    router.get("/", || async { Ok(Response::json_val(json!({"status": "ok"}))) });

    let mut server = test_router(router);
    let req = TestRequest::get("/").build_oxidite();
    let resp = server.call(req).await.unwrap();

    let test_resp = TestResponse::from_oxidite_response(resp).await;
    assert!(test_resp.is_success());
}
"#,
    };

    fs::write(path.join("tests/integration_test.rs"), test_content)
}

// ---------------------------------------------------------------------------
// Cargo.toml
// ---------------------------------------------------------------------------

fn create_cargo_toml(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    // Dependency version for consumer projects — always points to the latest
    // published release on crates.io, not the CLI's own version.
    const OXIDITE_VERSION: &str = "2.3.4";
    let (features_comment, features_list, extra_deps) = match p_type {
        ProjectType::Fullstack => (
            "full — everything included",
            r#"features = ["full"]"#,
            "",
        ),
        ProjectType::Api => (
            "all features except templates (use fullstack if you need HTML)",
            r#"default-features = false, features = ["database", "auth", "queue", "cache", "realtime", "mail", "storage", "graphql", "plugin", "security", "utils"]"#,
            "",
        ),
        ProjectType::Microservice => (
            "all features except templates (use fullstack if you need HTML)",
            r#"default-features = false, features = ["database", "auth", "queue", "cache", "realtime", "mail", "storage", "graphql", "plugin", "security", "utils"]"#,
            r#"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }"#,
        ),
        ProjectType::Serverless => (
            "minimal — add features as needed (database, auth, queue, ...)",
            r#"default-features = false, features = ["minimal"]"#,
            r#"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }"#,
        ),
    };

    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
# Oxidite — only the features this project type needs: {features_comment}.
# To add more: oxidite = {{ features = ["auth", "realtime", ...] }}
        oxidite = {{ version = "{OXIDITE_VERSION}", {features_list} }}

tokio = {{ version = "1", features = ["full"] }}

serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"{extra_deps}

[dev-dependencies]
oxidite-testing = "{OXIDITE_VERSION}"
"#,
        name = name,
        OXIDITE_VERSION = OXIDITE_VERSION,
        features_comment = features_comment,
        features_list = features_list,
        extra_deps = extra_deps,
    );

    fs::write(path.join("Cargo.toml"), content)
}

// ---------------------------------------------------------------------------
// README — overview, structure, quick-start, and deployment links
// ---------------------------------------------------------------------------

fn create_readme(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    let project_kind = match p_type {
        ProjectType::Fullstack => "fullstack application",
        ProjectType::Api => "API service",
        ProjectType::Microservice => "microservice",
        ProjectType::Serverless => "serverless function",
    };

    let structure = match p_type {
        ProjectType::Serverless => r#"├── src/
│   ├── handlers/     # One file per function (api, event, cron)
│   ├── core/         # Shared domain types and business logic
│   ├── config/       # Environment-aware config loader
│   └── main.rs       # Local dev server (not used in production)
├── infra/
│   ├── aws/          # Terraform: API Gateway, Lambda, IAM
│   └── cloudflare/   # wrangler.toml for Workers deploy
├── config/           # Per-environment TOML configs
├── Cargo.toml        # Minimal deps (no template, mail, storage)
├── oxidite.toml      # Framework + provider config
└── .env.example"#,
        ProjectType::Microservice => r#"├── src/
│   ├── main.rs        # Entry point with HTTP + queue startup
│   ├── handlers/      # HTTP handlers (thin, delegate to core)
│   ├── core/          # Domain logic and business rules
│   ├── events/        # Event publishers / subscribers
│   ├── queues/        # Message consumer workers
│   ├── models/        # Data / domain models
│   └── config/        # Service configuration
├── oxidite.toml       # Framework configuration
├── migrations/        # SQL migration files
├── seeds/             # Database seed files
└── tests/             # Integration tests"#,
        _ => r#"├── src/
│   ├── main.rs        # Application entry point
│   ├── routes/        # Route registration
│   ├── controllers/   # Request handlers
│   ├── models/        # Data models and ORM structs
│   ├── services/      # Business logic
│   ├── middleware/    # Custom middleware
│   ├── validators/    # Input validation
│   ├── jobs/          # Background jobs
│   ├── policies/      # Authorization policies
│   └── events/        # Event handlers
├── oxidite.toml       # Framework configuration
├── migrations/        # SQL migration files
├── seeds/             # Database seed files
└── tests/             # Integration tests"#,
    };

    let deploy_targets = match p_type {
        ProjectType::Serverless => r#"| `oxidite deploy --target aws-lambda` | AWS Lambda + API Gateway |
| `oxidite deploy --target cloudflare` | Cloudflare Workers |
| `oxidite deploy --target docker`     | Container (Lambda, Cloud Run, ECS) |
| Manual deploy                        | See `DEPLOYMENT.md` for Google Cloud Functions, Azure Functions"#,
        ProjectType::Fullstack => r#"| Docker                    | `docker build -t <name> . && docker run -p 8080:8080 <name>` |
| AWS ECS / App Runner      | See `DEPLOYMENT.md`                                            |
| Google Cloud Run           | See `DEPLOYMENT.md`                                            |
| Fly.io                     | `fly launch`                                                    |
| Manual VPS (DigitalOcean)  | See `DEPLOYMENT.md`                                            |"#,
        ProjectType::Api => r#"| Docker                    | `docker build -t <name> . && docker run -p 8080:8080 <name>` |
| AWS ECS / App Runner      | See `DEPLOYMENT.md`                                            |
| Google Cloud Run           | See `DEPLOYMENT.md`                                            |
| Fly.io / Railway           | `fly launch` / `railway up`                                    |
| Kubernetes                 | See `DEPLOYMENT.md`                                            |"#,
        ProjectType::Microservice => r#"| Docker                    | `docker build -t <name> . && docker run -p 8080:8080 <name>` |
| AWS ECS / Fargate         | See `DEPLOYMENT.md`                                            |
| Google Cloud Run           | See `DEPLOYMENT.md`                                            |
| Kubernetes                 | See `DEPLOYMENT.md`                                            |
| Fly.io                     | `fly launch`                                                    |"#,
    };

    let content = format!(
        r#"# {name}

Generated by `oxidite new` as an Oxidite **{project_kind}**.

## Quick Start

```bash
cargo install oxidite-cli
{quick_start_cmds}
```

## Project Structure

```
{structure}
```

## Deployment

Deploy anywhere — containers, serverless, or bare metal.

| Target                  | Quick Command / Guide              |
|-------------------------|-------------------------------------|
{deploy_targets}

See **`DEPLOYMENT.md`** for step-by-step guides on every platform.

## Commands

| Command              | Description                          |
|----------------------|--------------------------------------|
| `oxidite dev`        | Start dev server with hot-reload     |
| `oxidite doctor`     | System health check                  |
| `oxidite deploy`     | Generate deployment artifacts        |
| {migrate_cmd}
| `oxidite make model` | Scaffold a new model                 |
"#,
        name = name,
        project_kind = project_kind,
        quick_start_cmds = match p_type {
            ProjectType::Serverless => "oxidite dev",
            _ => "oxidite migrate\noxidite dev",
        },
        structure = structure,
        deploy_targets = deploy_targets,
        migrate_cmd = match p_type {
            ProjectType::Serverless => "`oxidite run script.rs` | Run a standalone Rust script      |",
            _ => "`oxidite migrate`    | Run pending migrations               |",
        },
    );

    fs::write(path.join("README.md"), content)
}

// ---------------------------------------------------------------------------
// DEPLOYMENT.md — platform-specific guides for every project type
// ---------------------------------------------------------------------------

fn create_deployment_doc(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    let build_section = match p_type {
        ProjectType::Serverless => r#"### Build the binary

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

For ARM/Graviton:
```bash
cargo build --release --target aarch64-unknown-linux-musl
```
"#,
        _ => r#"### Build the binary

```bash
cargo build --release
```"#,
    };

    let deploy_guides = match p_type {
        ProjectType::Serverless => r#"## Docker (any platform)

```dockerfile
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/PROJECT_NAME /bootstrap
CMD ["/bootstrap"]
```

```bash
docker build -t PROJECT_NAME:latest .
docker run -e OXIDITE_SERVERLESS=1 -p 8080:8080 PROJECT_NAME:latest
```

## AWS Lambda

Uses the Lambda Custom Runtime API. Package as a container image:

```bash
# Build for Lambda's Amazon Linux
cargo build --release --target x86_64-unknown-linux-musl

# Create bootstrap zip (for zip-based Lambda, not container)
cp target/x86_64-unknown-linux-musl/release/PROJECT_NAME bootstrap
zip lambda.zip bootstrap
```

For container-based Lambda, use the Dockerfile in `infra/aws/` and push to ECR.

## Google Cloud Run

```dockerfile
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/PROJECT_NAME /server
CMD ["/server"]
```

```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/PROJECT_NAME
gcloud run deploy PROJECT_NAME --image gcr.io/PROJECT_ID/PROJECT_NAME --port 8080
```

## Cloudflare Workers

See `infra/cloudflare/wrangler.toml` and run:

```bash
npx wrangler deploy
```
"#,
        ProjectType::Fullstack => r#"## Docker

```dockerfile
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY public ./public
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/PROJECT_NAME /app/server
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public /app/public
WORKDIR /app
EXPOSE 8080
CMD ["/app/server"]
```

```bash
docker build -t PROJECT_NAME:latest .
docker run -p 8080:8080 -e SERVER_HOST=0.0.0.0 PROJECT_NAME:latest
```

## AWS ECS (Fargate)

1. Push the Docker image to Amazon ECR.
2. Create an ECS task definition (256 MB memory, 0.25 vCPU is sufficient for most Rust apps).
3. Deploy as an ECS service with Fargate launch type behind an Application Load Balancer.
4. Set environment variables in the task definition: `SERVER_HOST=0.0.0.0`, `OXIDITE_ENV=production`.

## Google Cloud Run

```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/PROJECT_NAME
gcloud run deploy PROJECT_NAME --image gcr.io/PROJECT_ID/PROJECT_NAME --port 8080
```

## Fly.io

```bash
# Automatic Dockerfile detection
fly launch --name PROJECT_NAME
fly deploy
```

Or use a `fly.toml`:

```toml
app = "PROJECT_NAME"

[build]
  docker = "Dockerfile"

[[services]]
  internal_port = 8080
  protocol = "tcp"
  [[services.ports]]
    handlers = ["http"]
    port = 80
```

## Manual VPS (DigitalOcean, Hetzner, etc.)

```bash
# On your dev machine
cargo build --release
scp target/release/PROJECT_NAME user@host:/opt/PROJECT_NAME/server

# On the VPS — create a systemd service
cat > /etc/systemd/system/PROJECT_NAME.service << 'EOF'
[Unit]
Description=PROJECT_NAME Oxidite server

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/PROJECT_NAME
ExecStart=/opt/PROJECT_NAME/server
Environment=SERVER_HOST=0.0.0.0
Environment=SERVER_PORT=8080
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl enable PROJECT_NAME --now
```
"#,
        _ => r#"## Docker

```dockerfile
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/PROJECT_NAME /app/server
EXPOSE 8080
CMD ["/app/server"]
```

```bash
docker build -t PROJECT_NAME:latest .
docker run -p 8080:8080 -e SERVER_HOST=0.0.0.0 PROJECT_NAME:latest
```

## AWS ECS / App Runner

1. Push the Docker image to Amazon ECR.
2. For **App Runner**: connect your ECR repo, set port 8080, configure environment variables in the console.
3. For **ECS/Fargate**: create a task definition with 256 MB memory, deploy as a service behind ALB.

## Google Cloud Run

```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/PROJECT_NAME
gcloud run deploy PROJECT_NAME --image gcr.io/PROJECT_ID/PROJECT_NAME --port 8080
```

## Fly.io

```bash
fly launch --name PROJECT_NAME
fly deploy
```

## Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: PROJECT_NAME
spec:
  replicas: 2
  selector:
    matchLabels:
      app: PROJECT_NAME
  template:
    metadata:
      labels:
        app: PROJECT_NAME
    spec:
      containers:
      - name: server
        image: PROJECT_NAME:latest
        ports:
        - containerPort: 8080
        env:
        - name: SERVER_HOST
          value: "0.0.0.0"
        - name: OXIDITE_ENV
          value: "production"
---
apiVersion: v1
kind: Service
metadata:
  name: PROJECT_NAME
spec:
  selector:
    app: PROJECT_NAME
  ports:
  - port: 8080
```

## Railway

```bash
railway login
railway init
# Set SERVER_HOST=0.0.0.0 and SERVER_PORT=8080 in Railway dashboard
railway up
```

## Manual VPS

```bash
cargo build --release
scp target/release/PROJECT_NAME user@host:/opt/PROJECT_NAME/server

# systemd service
cat > /etc/systemd/system/PROJECT_NAME.service << 'UNIT'
[Unit]
Description=PROJECT_NAME API server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/PROJECT_NAME
ExecStart=/opt/PROJECT_NAME/server
Environment=SERVER_HOST=0.0.0.0
Environment=SERVER_PORT=8080
Restart=always

[Install]
WantedBy=multi-user.target
UNIT
systemctl enable PROJECT_NAME --now
```
"#,
    };

    let content = format!(
        r#"# Deploying {name}

This guide covers deployment options for the {name} Oxidite project.

{build_section}
{deploy_guides}

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Bind address | `127.0.0.1` |
| `SERVER_PORT` | Port to listen on | `8080` |
| `OXIDITE_ENV` | Environment name | `development` |
| `DATABASE_URL` | Database connection string | — |
| `JWT_SECRET` | JWT signing secret | — |
| `REDIS_URL` | Redis connection string | — |

## Health Check

Configure your platform's health check to hit `GET /api/health` (or `/` for fullstack).
The expected success status is `200 OK`.

## Logging

By default, logs go to stdout/stderr in plain text. To enable structured JSON logging,
set `OXIDITE_LOG_FORMAT=json`.

## Production Checklist

- [ ] Set `OXIDITE_ENV=production`
- [ ] Generate a strong `JWT_SECRET`
- [ ] Use a managed database (RDS, Cloud SQL, etc.) — not SQLite in production
- [ ] Enable HTTPS via your platform's load balancer or reverse proxy
- [ ] Set up a monitoring/alerting solution
- [ ] Configure database connection pooling (max 10–20 connections per instance)
"#,
        name = name,
        build_section = build_section,
        deploy_guides = deploy_guides.replace("PROJECT_NAME", name),
    );

    fs::write(path.join("DEPLOYMENT.md"), content)
}

// ---------------------------------------------------------------------------
// oxidite.toml
// ---------------------------------------------------------------------------

fn create_config_toml(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let mut content = String::from(
        r#"# Oxidite Configuration
#
# Environment Variables — three strategies (all produce the same env::var() results):
#
# 1. [env] flat table — keys map directly to env vars:
#        [env]
#        DATABASE_URL = "postgres://..."
#        API_KEY = "secret"
#
# 2. Namespaced tables — table name becomes an UPPERCASE prefix:
#        [google]
#        client_id = "abc"        -> GOOGLE_CLIENT_ID
#        client_secret = "xyz"    -> GOOGLE_CLIENT_SECRET
#
# 3. Nested tables — flattened recursively:
#        [google.oauth]
#        client_id = "abc"        -> GOOGLE_OAUTH_CLIENT_ID
#
# .env files also work and take priority over oxidite.toml values.
# Real OS environment variables always take highest priority.
#
# All strategies are interchangeable — use whichever fits best.

[server]
host = "127.0.0.1"
port = 8080
"#,
    );

    match p_type {
        ProjectType::Fullstack | ProjectType::Api => {
            content.push_str(
                r#"
[database]
url = "sqlite://./data.db"

# Define env vars here or in a .env file. Examples:
#
# [env]
# JWT_SECRET = "change-me"
#
# [platform]
# name = "my-app"              -> PLATFORM_NAME
"#,
            );
        }
        ProjectType::Microservice => {
            content.push_str(
                r#"
[queue]
redis_url = "redis://localhost"

# Define env vars here or in a .env file. Examples:
#
# [env]
# JWT_SECRET = "change-me"
#
# [broker]
# url = "redis://localhost"    -> BROKER_URL
"#,
            );
        }
        ProjectType::Serverless => {
            content.push_str(
                r#"
# Serverless-specific configuration.
# Environment overrides live in config/dev.toml, config/staging.toml, config/prod.toml
# so they can be checked in safely without .env files.

[provider]
name = "aws"
region = "us-east-1"

# Each handler can be deployed as its own Lambda function.
# Define memory, timeout, and triggers per handler here.
[handlers.api]
memory = 512
timeout = 30
triggers = ["http"]

[handlers.event_processor]
memory = 1024
timeout = 60
triggers = ["sqs"]

[handlers.scheduled_task]
memory = 256
timeout = 120
triggers = ["cron"]

[env]
LOG_LEVEL = "info"
"#,
            );
        }
    }

    fs::write(path.join("oxidite.toml"), content)
}

// ---------------------------------------------------------------------------
// src/main.rs
// ---------------------------------------------------------------------------

fn create_main_rs(path: &Path, p_type: ProjectType) -> std::io::Result<()> {
    let content = match p_type {
        ProjectType::Fullstack => {
            r#"// Application entry point.
//
// This loads configuration, registers routes, applies middleware,
// and starts the HTTP server — in that order.
//
// ── Boot sequence ─────────────────────────────────────────────────────
//   Config  ──>  Router  ──>  Middleware  ──>  Server
// ──────────────────────────────────────────────────────────────────────
//
// Two styles are supported:
//
//   1. Application builder (recommended):
//        Application::new(config)  →  app.router_mut().get(...)  →  app.run()
//
//   2. Manual (drop-in replacement, same APIs as before):
//        Router::new()  →  router.get(...)  →  Server::new(router).listen(addr)
//
// Both produce the same result. Mix and match freely.

use oxidite::prelude::*;
use oxidite::template::serve_static;

// Each module maps to a directory under `src/`.
// Use `oxidite make <kind> <name>` to scaffold new modules.
mod routes;
mod controllers;
mod middleware;
mod models;
mod services;
mod validators;
mod jobs;
mod policies;
mod events;

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Config ──────────────────────────────────────────────────────
    // Load configuration from oxidite.toml, .env, and OS env overrides.
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    // ── 2. Router ──────────────────────────────────────────────────────
    // Application coordinates the boot sequence automatically.
    // It reads server.host / server.port from Config when you call run().
    let mut app = Application::new(config);
    routes::register(app.router_mut());

    // Serve static files from `public/` as a catch-all fallback.
    //   GET /css/style.css    ->  public/css/style.css
    //   GET /images/logo.svg  ->  public/images/logo.svg
    app.router_mut().get("/*", serve_static);

    // ── 3. Middleware ──  4. Server ────────────────────────────────────
    // To apply middleware (e.g. Logger), extract the router first:
    //   let router = app.into_router();
    //   let server = Server::new(Logger::new(router));
    //
    // For simple cases without middleware, app.run() is all you need:
    println!("Server running on http://{}:{}",
        app.config().server.host, app.config().server.port);
    app.run().await
}
"#
        }
        ProjectType::Microservice => {
            r#"// Microservice entry point.
//
// Focused, independently-deployable service with HTTP handlers,
// event consumers, and queue workers. No MVC boilerplate.
//
// ── Boot sequence ─────────────────────────────────────────────────────
//   Config  ──>  Router  ──>  Middleware  ──>  Server
// ──────────────────────────────────────────────────────────────────────

use oxidite::prelude::*;
use tracing::info;

mod handlers;
mod core;
mod events;
mod queues;
mod models;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // ── 1. Config ──────────────────────────────────────────────────────
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    // Start background queue consumers before accepting HTTP traffic.
    queues::register().await;

    // ── 2. Router ──────────────────────────────────────────────────────
    let mut app = Application::new(config);
    handlers::routes(app.router_mut());

    // ── 3. Middleware ──  4. Server ────────────────────────────────────
    info!("Microservice listening on http://{}:{}",
        app.config().server.host, app.config().server.port);
    app.run().await
}
"#
        }
        ProjectType::Api => {
            r#"// API service entry point.
//
// ── Boot sequence ─────────────────────────────────────────────────────
//   Config  ──>  Router  ──>  Middleware  ──>  Server
// ──────────────────────────────────────────────────────────────────────
//
// Application builder (recommended):
//   Application::new(config)  →  app.router_mut().get(...)  →  app.run()
//
// Manual equivalent (still works):
//   Router::new()  →  router.get(...)  →  Server::new(router).listen(addr)

use oxidite::prelude::*;

mod routes;
mod controllers;
mod middleware;
mod models;
mod services;
mod validators;
mod jobs;
mod policies;
mod events;

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Config ──────────────────────────────────────────────────────
    let config = Config::load()
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let host = config.server.host.clone();
    let port = config.server.port;

    // ── 2. Router ──────────────────────────────────────────────────────
    let mut app = Application::new(config);
    routes::register(app.router_mut());

    // ── 3. Middleware ──  4. Server ────────────────────────────────────
    // With middleware: let router = app.into_router();
    //                 Server::new(Logger::new(router)).listen(addr).await
    //
    // Without middleware, app.run() reads host/port from Config:
    println!("Server running on http://{host}:{port}");
    app.run().await
}
"#
        }
        ProjectType::Serverless => {
            r#"// Oxidite Serverless Function — entry point.
//
// Handlers live in src/handlers/ — each file can be independently deployed.
// main() is for local development only; production uses OXIDITE_SERVERLESS=1
// and a cloud runtime adapter (Lambda custom runtime, etc.).
//
// ── Boot sequence ─────────────────────────────────────────────────────
//   Config  ──>  Router  ──>  Middleware  ──>  Server
// ──────────────────────────────────────────────────────────────────────

mod handlers;
mod core;
mod config;

use oxidite::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Production mode: the cloud runtime adapter is expected to invoke
    // handlers::api::handle or other handler fns directly. We just block.
    if std::env::var("OXIDITE_SERVERLESS").is_ok() {
        info!("Serverless function initialised — waiting for runtime adapter");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }

    info!("Starting local development server");

    // ── 2. Router ──────────────────────────────────────────────────────
    let config = oxidite::config::Config::default();
    let mut app = Application::new(config);

    // Mount each HTTP handler under its route for local testing.
    app.router_mut().get("/api/hello", handlers::api::handle);

    // Event and cron handlers have no HTTP trigger locally;
    // call them from tests or via the tinker REPL.

    // ── 3. Middleware ──  4. Server ────────────────────────────────────
    app.run().await
}
"#
        }
    };

    fs::write(path.join("src/main.rs"), content)
}

// ---------------------------------------------------------------------------
// Boilerplate files (routes, modules, static assets, templates)
// ---------------------------------------------------------------------------

fn create_boilerplate(path: &Path, name: &str, p_type: ProjectType) -> std::io::Result<()> {
    if matches!(p_type, ProjectType::Serverless) {
        return create_serverless_boilerplate(path, name);
    }

    if matches!(p_type, ProjectType::Microservice) {
        return create_microservice_boilerplate(path);
    }

    // Standard non-serverless: module barrels, routes, assets.
    let module_files: &[(&str, &str)] = &[
        ("src/models/mod.rs",
         "// Data models and ORM structs are defined here.\n\
          // Use `oxidite make model User name:string email:string` to scaffold.\n"),
        ("src/controllers/mod.rs",
         "// Controllers handle HTTP requests and return responses.\n\
          // Use `oxidite make controller Auth` to scaffold.\n"),
        ("src/services/mod.rs",
         "// Services contain business logic, keeping controllers thin.\n\
          // Use `oxidite make service Payment` to scaffold.\n"),
        ("src/middleware/mod.rs",
         "// Middleware runs before/after request handlers.\n\
          // Use `oxidite make middleware RequireAuth` to scaffold.\n"),
        ("src/validators/mod.rs",
         "// Validators check and sanitize user input.\n\
          // Use `oxidite make validator CreateUser` to scaffold.\n"),
        ("src/jobs/mod.rs",
         "// Background jobs for async processing (emails, reports, etc.).\n\
          // Use `oxidite make job SendWelcomeEmail` to scaffold.\n"),
        ("src/policies/mod.rs",
         "// Authorization policies control access to resources.\n\
          // Use `oxidite make policy PostPolicy` to scaffold.\n"),
        ("src/events/mod.rs",
         "// Event handlers react to domain events.\n\
          // Use `oxidite make event UserRegistered` to scaffold.\n"),
    ];

    for (file_path, content) in module_files {
        fs::write(path.join(file_path), content)?;
    }

    // Routes — register all HTTP handlers here.
    let routes_content = match p_type {
        ProjectType::Fullstack => {
            r#"// Route registration.
//
// Add your routes inside `register()`. The `register_generated()` function
// is where `oxidite make route` inserts new route bindings automatically.
//
// Template rendering:
//   This scaffold registers a TemplateContext as shared state so that
//   every handler can render templates without creating the engine
//   from scratch. See the `index` handler below.
//
//   If you prefer the manual per-request approach (no state), just
//   drop the `router.with_state(...)` call and create a TemplateEngine
//   directly inside each handler — both patterns work.

use oxidite::prelude::*;
use oxidite::template::{Context, TemplateContext};
use std::sync::Arc;

/// Register all application routes on the router.
pub fn register(router: &mut Router) {
    // Share template directory config via type-safe State extractor.
    // The engine is created per-call — zero global lifecycle coupling.
    let templates = Arc::new(TemplateContext::new("templates"));
    router.with_state(templates);

    router.get("/", index);
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
}

async fn index(templates: State<Arc<TemplateContext>>) -> Result<Response> {
    let mut context = Context::new();
    context.set("name", "Oxidite");

    // State is a tuple struct — deref with `.0` to access the inner TemplateContext
    let body = templates.0
        .render("index.html", &context)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    Ok(Response::html(body))
}
"#
        }
        ProjectType::Api => {
            r#"// Route registration.

use oxidite::prelude::*;

pub fn register(router: &mut Router) {
    router.get("/api/health", health);
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
}

async fn health(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({"status": "ok"})))
}
"#
        }
        _ => {
            r#"// Route registration.

use oxidite::prelude::*;

pub fn register(router: &mut Router) {
    register_generated(router);
}

fn register_generated(router: &mut Router) {
    let _ = router;
}
"#
        }
    };
    fs::write(path.join("src/routes/mod.rs"), routes_content)?;

    // Fullstack-specific: static assets, logo, and HTML template.
    if let ProjectType::Fullstack = p_type {
        // ---- CSS (with comments for beginners) ----
        let css_content = r#"/* Oxidite default styles.
 *
 * These styles create a simple centred card layout for the welcome page.
 * Replace or extend them to match your design.
 */

body {
    /* System font stack for fast, native-looking text. */
    font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;

    /* Centre the card both horizontally and vertically. */
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    margin: 0;

    /* Dark background. */
    background-color: #0f172a;
    color: #e2e8f0;
}

.container {
    text-align: center;
    padding: 40px;
    background-color: #1e293b;
    border-radius: 12px;
}

/* Logo sizing — adjust width/height to taste. */
.logo {
    width: 200px;
    height: auto;
    margin-bottom: 24px;
}

h1 {
    margin: 0 0 8px;
    font-size: 2rem;
}

p {
    margin: 0;
    opacity: 0.8;
}
"#;
        fs::write(path.join("public/css/style.css"), css_content)?;

        // ---- JS placeholder ----
        let js_content = r#"// Client-side JavaScript.
// This file is served at /js/app.js from the public/ directory.
console.log('Oxidite app loaded');
"#;
        fs::write(path.join("public/js/app.js"), js_content)?;

        // ---- Oxidite SVG logo ----
        // Written as a static file in public/images/.
        // Referenced in HTML as <img src="/images/oxidite.svg">.
        // Developers can delete or replace this file freely.
        fs::write(path.join("public/images/oxidite.svg"), OXIDITE_LOGO_SVG)?;

        // ---- HTML welcome template ----
        // Uses Handlebars-style {{ variable }} placeholders.
        // The logo and favicon both reference files in public/.
        let template_content = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Hello, Oxidite</title>

  <!-- Favicon — the Oxidite logo from public/images/.
       Delete this line or point it to your own icon. -->
  <link rel="icon" type="image/svg+xml" href="/images/oxidite.svg">

  <!-- Styles from public/css/style.css -->
  <link rel="stylesheet" href="/css/style.css">
</head>
<body>
  <div class="container">
    <!-- Logo — served from public/images/oxidite.svg.
         Remove this <img> or replace the src with your own logo. -->
    <img class="logo" src="/images/oxidite.svg" alt="Oxidite logo">

    <h1>Hello, {{ name }}!</h1>
    <p>Your Oxidite app is running. Start building something great.</p>
  </div>

  <!-- Client-side JS from public/js/app.js -->
  <script src="/js/app.js"></script>
</body>
</html>
"#;
        fs::write(path.join("templates/index.html"), template_content)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Microservice boilerplate — handlers, events, queues, core
// ---------------------------------------------------------------------------

fn create_microservice_boilerplate(path: &Path) -> std::io::Result<()> {
    // ── src/handlers/mod.rs ───────────────────────────────────────────
    fs::write(
        path.join("src/handlers/mod.rs"),
        r#"pub mod health;

use oxidite::prelude::*;

pub fn routes(router: &mut Router) {
    router.get("/health", health::check);
    router.get("/api/v1/status", health::status);
}
"#,
    )?;

    // ── src/handlers/health.rs ───────────────────────────────────────
    fs::write(
        path.join("src/handlers/health.rs"),
        r#"use oxidite::prelude::*;

pub async fn check(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({"status": "ok"})))
}

pub async fn status(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({
        "service": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "status": "ok"
    })))
}
"#,
    )?;

    // ── src/core/mod.rs — domain logic ────────────────────────────────
    fs::write(
        path.join("src/core/mod.rs"),
        r#"// Domain types and business logic shared across handlers and events.
"#,
    )?;

    // ── src/events/mod.rs — message publishing / subscribing ──────────
    fs::write(
        path.join("src/events/mod.rs"),
        r#"// Event publishers and subscribers for async messaging.
// Examples: SQS, EventBridge, RabbitMQ, Kafka, NATS.
//
// pub mod order_events;
// pub mod notification_events;
"#,
    )?;

    // ── src/queues/mod.rs — message queue consumers ───────────────────
    fs::write(
        path.join("src/queues/mod.rs"),
        r#"use tracing::info;

/// Register and start background queue consumers.
/// Called once at service startup before the HTTP server binds.
pub async fn register() {
    info!("No queue consumers registered");
}

// Example consumer:
// pub async fn process_order_events() {
//     let mut rx = channel.subscribe("orders.created").await;
//     while let Some(msg) = rx.recv().await {
//         info!("Received order event: {:?}", msg);
//     }
// }
"#,
    )?;

    // ── src/models/mod.rs — data / domain models ──────────────────────
    fs::write(
        path.join("src/models/mod.rs"),
        r#"// Data models and ORM structs.
// Use `oxidite make model <name> <field>:<type>` to scaffold.
"#,
    )?;

    // ── src/config/mod.rs — service config ────────────────────────────
    fs::write(
        path.join("src/config/mod.rs"),
        r#"// Microservice-specific configuration.
// Import and use shared config::Config from oxidite.
pub use oxidite::config::Config;
"#,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Serverless boilerplate — handlers, core, infra-as-code, env configs
// ---------------------------------------------------------------------------

fn create_serverless_boilerplate(path: &Path, name: &str) -> std::io::Result<()> {
    // ── src/handlers/mod.rs ────────────────────────────────────────────
    fs::write(
        path.join("src/handlers/mod.rs"),
        r#"pub mod api;
pub mod event;
pub mod cron;
"#,
    )?;

    // ── src/handlers/api.rs — HTTP request handler ────────────────────
    fs::write(
        path.join("src/handlers/api.rs"),
        r#"use oxidite::prelude::*;

/// HTTP API handler — deploy as Lambda Function URL, API Gateway, etc.
/// Accessible locally at GET /api/hello
pub async fn handle(_req: Request) -> Result<Response> {
    Ok(Response::json_val(json!({
        "service": "oxidite-serverless",
        "message": "Hello from the API handler"
    })))
}
"#,
    )?;

    // ── src/handlers/event.rs — Event-driven handler ──────────────────
    fs::write(
        path.join("src/handlers/event.rs"),
        r#"use serde::{Deserialize, Serialize};

/// Example domain event consumed from SQS / EventBridge / Kafka.
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderCreated {
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
}

/// Event handler — triggered asynchronously by a message bus.
/// In production this is invoked by a Lambda event-source mapping.
pub async fn handle_event(event: OrderCreated) -> Result<(), String> {
    tracing::info!(
        "Processing order {}: {} {}",
        event.order_id,
        event.amount,
        event.currency,
    );
    Ok(())
}
"#,
    )?;

    // ── src/handlers/cron.rs — Scheduled / CRON handler ───────────────
    fs::write(
        path.join("src/handlers/cron.rs"),
        r#"/// Scheduled task handler — triggered by EventBridge Scheduler / cron().
/// In production each cron expression maps to its own Lambda invocation.
pub async fn handle_scheduled() -> Result<(), String> {
    tracing::info!("Scheduled task running");
    Ok(())
}
"#,
    )?;

    // ── src/core/mod.rs — shared domain logic ─────────────────────────
    fs::write(
        path.join("src/core/mod.rs"),
        r#"// Shared business logic, domain types, and helpers.
// Handlers import from here instead of duplicating code.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "ok".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}
"#,
    )?;

    // ── src/config/mod.rs — environment-aware config ──────────────────
    fs::write(
        path.join("src/config/mod.rs"),
        r#"use oxidite::config::Config;
use std::env;

/// Load configuration with environment-aware overrides.
/// Tries `config/{ENV}.toml` first, falls back to `oxidite.toml`.
pub fn load() -> Config {
    let env_name = env::var("OXIDITE_ENV").unwrap_or_else(|_| "dev".into());

    let config_path = format!("config/{}.toml", env_name);
    if std::path::Path::new(&config_path).exists() {
        Config::load_from(&config_path).unwrap_or_default()
    } else {
        Config::load().unwrap_or_default()
    }
}
"#,
    )?;

    // ── infra/aws/main.tf ─────────────────────────────────────────────
    let tf_content = r#"# Terraform for PROJECT_NAME — Oxidite serverless functions.

terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

variable "app_name" {
  default = "PROJECT_NAME"
}

variable "aws_region" {
  default = "us-east-1"
}

resource "aws_apigatewayv2_api" "http" {
  name          = "${var.app_name}-api"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.http.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_iam_role" "lambda_exec" {
  name = "${var.app_name}-exec"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "basic" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function" "api" {
  function_name = "${var.app_name}-api"
  package_type  = "Image"
  image_uri     = "${var.app_name}:latest"
  role          = aws_iam_role.lambda_exec.arn
  timeout       = 30
  memory_size   = 512
  environment {
    variables = {
      OXIDITE_SERVERLESS = "1"
      OXIDITE_ENV        = "production"
    }
  }
}

resource "aws_lambda_permission" "api_gw" {
  function_name = aws_lambda_function.api.function_name
  action        = "lambda:InvokeFunction"
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http.execution_arn}/*/*"
}

resource "aws_apigatewayv2_integration" "lambda" {
  api_id             = aws_apigatewayv2_api.http.id
  integration_type   = "AWS_PROXY"
  integration_uri    = aws_lambda_function.api.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "any" {
  api_id    = aws_apigatewayv2_api.http.id
  route_key = "ANY /{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.lambda.id}"
}
"#.replace("PROJECT_NAME", name);
    fs::write(path.join("infra/aws/main.tf"), tf_content)?;

    // ── infra/aws/variables.tf ────────────────────────────────────────
    fs::write(
        path.join("infra/aws/variables.tf"),
        r#"variable "environment" {
  description = "Deployment environment (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "tags" {
  description = "Common resource tags"
  type        = map(string)
  default = {
    Terraform   = "true"
    Framework   = "oxidite"
  }
}
"#,
    )?;

    // ── infra/cloudflare/wrangler.toml ────────────────────────────────
    let wrangler_content = format!(
        r#"name = "{}"
main = "src/main.rs"
compatibility_date = "2025-01-01"

[build]
command = "cargo build --release --target wasm32-wasi"
watch_dir = "src"

[[build.assets]]
pattern = "*.wasm"
"#,
        name
    );
    fs::write(path.join("infra/cloudflare/wrangler.toml"), wrangler_content)?;

    // ── config/dev.toml, staging.toml, prod.toml ──────────────────────
    let env_configs = [
        ("dev.toml",
         r#"[server]
host = "127.0.0.1"
port = 8080

[env]
LOG_LEVEL = "debug"
"#,
        ),
        ("staging.toml",
         r#"[server]
host = "0.0.0.0"
port = 8080

[provider]
region = "eu-west-1"

[env]
LOG_LEVEL = "info"
"#,
        ),
        ("prod.toml",
         r#"[server]
host = "0.0.0.0"
port = 8080

[provider]
region = "us-east-1"

[handlers.api]
memory = 1024
timeout = 30

[handlers.event_processor]
memory = 2048
timeout = 120

[env]
LOG_LEVEL = "warn"
"#,
        ),
    ];

    for (name, content) in &env_configs {
        fs::write(path.join("config").join(name), content)?;
    }

    // ── .env.example ──────────────────────────────────────────────────
    fs::write(
        path.join(".env.example"),
        r#"OXIDITE_ENV=dev
LOG_LEVEL=debug
# AWS_ACCESS_KEY_ID=
# AWS_SECRET_ACCESS_KEY=
# AWS_REGION=us-east-1
# DATABASE_URL=postgres://...
# REDIS_URL=redis://...
"#,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — verify that generated projects compile for every project type
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_type_arg(p_type: ProjectType) -> &'static str {
        match p_type {
            ProjectType::Fullstack => "fullstack",
            ProjectType::Api => "api",
            ProjectType::Microservice => "microservice",
            ProjectType::Serverless => "serverless",
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("CARGO_MANIFEST_DIR parent")
            .to_path_buf()
    }

    fn patch_section() -> String {
        let root = workspace_root();
        let crates = [
            "oxidite",
            "oxidite-testing",
            "oxidite-core",
            "oxidite-config",
            "oxidite-middleware",
            "oxidite-db",
            "oxidite-auth",
            "oxidite-queue",
            "oxidite-cache",
            "oxidite-realtime",
            "oxidite-template",
            "oxidite-mail",
            "oxidite-storage",
            "oxidite-macros",
            "oxidite-utils",
            "oxidite-openapi",
            "oxidite-graphql",
            "oxidite-plugin",
            "oxidite-security",
        ];
        let mut section = String::from("\n[patch.crates-io]\n");
        for krate in &crates {
            let path = root.join(krate).display().to_string();
            section.push_str(&format!("{krate} = {{ path = \"{path}\" }}\n"));
        }
        section
    }

    fn run_test(p_type: ProjectType) {
        let dir = tempfile::tempdir().expect("tempdir");
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(dir.path()).expect("chdir");

        let name = match p_type {
            ProjectType::Fullstack => "test-fullstack",
            ProjectType::Api => "test-api",
            ProjectType::Microservice => "test-microservice",
            ProjectType::Serverless => "test-serverless",
        };

        create_project(name, Some(project_type_arg(p_type).into()), None, &[])
            .expect("create_project should succeed");

        let cargo_path = dir.path().join(name).join("Cargo.toml");
        let mut content = std::fs::read_to_string(&cargo_path).expect("read Cargo.toml");
        content.push_str(&patch_section());
        std::fs::write(&cargo_path, content).expect("write patched Cargo.toml");

        let output = std::process::Command::new("cargo")
            .args(["check"])
            .current_dir(dir.path().join(name))
            .output()
            .expect("cargo check");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        std::env::set_current_dir(original).expect("restore cwd");

        assert!(
            output.status.success(),
            "cargo check failed for {p_type:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }

    #[test]
    fn test_fullstack_project_compiles() {
        run_test(ProjectType::Fullstack);
    }

    #[test]
    fn test_api_project_compiles() {
        run_test(ProjectType::Api);
    }

    #[test]
    fn test_microservice_project_compiles() {
        run_test(ProjectType::Microservice);
    }

    #[test]
    fn test_serverless_project_compiles() {
        run_test(ProjectType::Serverless);
    }
}
