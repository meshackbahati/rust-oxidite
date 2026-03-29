# Docker Deployment Guide

This chapter covers Docker-based development and production deployment for Oxidite apps.

## Goals

- reproducible local dev environments
- predictable production images
- safe rollout and rollback using containers

## 1. Basic Dockerfile (single-stage)

```dockerfile
FROM rust:1.89-bookworm
WORKDIR /app

COPY . .
RUN cargo build --release

EXPOSE 8080
CMD ["./target/release/example-project"]
```

Use this for quick experiments only. It produces large images.

## 2. Recommended multi-stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.89-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY public ./public
COPY oxidite.toml ./

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/example-project /app/example-project
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/public /app/public
COPY --from=builder /app/oxidite.toml /app/oxidite.toml

EXPOSE 8080
CMD ["/app/example-project"]
```

## 3. .dockerignore baseline

```dockerignore
target
.git
.github
.DS_Store
*.log
.env
```

## 4. Docker Compose for local development

```yaml
services:
  app:
    build: .
    image: oxidite-example:dev
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info
    volumes:
      - ./templates:/app/templates:ro
      - ./public:/app/public:ro
```

Run:

```bash
docker compose up --build
```

## 5. Health checks

Expose an app health endpoint and add container health check:

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 15s
  timeout: 3s
  retries: 5
```

## 6. Production hardening checklist

- run as non-root user
- pin base image tags
- set CPU/memory limits
- configure restart policy
- forward logs to centralized collector
- externalize secrets (do not bake into image)

## 7. Performance considerations

- compile in release mode
- keep runtime image slim
- keep static assets separate when possible (CDN/reverse proxy)
- prefer immutable container images per release

## 8. Deployment patterns

- single VM with Docker Compose
- Kubernetes deployment + service + ingress
- blue/green or canary rollouts

## 9. Troubleshooting

- app exits immediately: verify binary path and execute permissions
- 404 for templates/static: verify copied paths and working directory
- slow startup: check image size and cold storage pulls
- TLS issues: terminate TLS at ingress/reverse proxy first

## 10. Suggested CI pipeline

1. `cargo check --workspace`
2. `cargo test --workspace`
3. build image
4. run container smoke test (`curl /health`)
5. push image
6. deploy with rollback metadata
