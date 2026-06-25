use std::fs;
use std::path::Path;

use super::output;

/// Generate deployment artifacts for serverless/container platforms.
pub fn generate_artifacts(target: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out = Path::new(output_dir);
    fs::create_dir_all(out)?;

    match target {
        "aws-lambda" => generate_aws_lambda(out),
        "docker" => generate_docker(out),
        "cloudflare" => generate_cloudflare(out),
        _ => Err(format!("Unknown deploy target: {}. Options: aws-lambda, docker, cloudflare", target).into()),
    }
}

fn generate_aws_lambda(out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    output::step("Generating AWS Lambda deployment artifacts");

    // Dockerfile for AWS Lambda custom runtime on Amazon Linux 2023
    let dockerfile = r#"FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM public.ecr.aws/lambda/provided:al2023
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/<YOUR_PROJECT> /bootstrap
CMD ["handler"]
"#;
    fs::write(out.join("Dockerfile"), dockerfile)?;
    output::info(&format!("  Wrote {}", out.join("Dockerfile").display()));

    // Build & deploy script
    let build_script = r#"#!/usr/bin/env bash
set -euo pipefail

ACCOUNT_ID="${AWS_ACCOUNT_ID:-$(aws sts get-caller-identity --query Account --output text)}"
REGION="${AWS_REGION:-us-east-1}"
FUNCTION_NAME="${1:-my-oxidite-function}"

echo "Building Lambda image..."
docker build --platform linux/amd64 \
  -t "${FUNCTION_NAME}:latest" .

echo "Tagging and pushing to ECR..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

REPO_URI="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${FUNCTION_NAME}"
docker tag "${FUNCTION_NAME}:latest" "${REPO_URI}:latest"
docker push "${REPO_URI}:latest"

echo "Updating Lambda function..."
aws lambda update-function-code \
  --function-name "$FUNCTION_NAME" \
  --image-uri "${REPO_URI}:latest" \
  --region "$REGION"

echo "Deployment complete."
"#;
    fs::write(out.join("deploy-aws.sh"), build_script)?;
    // make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(out.join("deploy-aws.sh"), fs::Permissions::from_mode(0o755))?;
    }
    output::info(&format!("  Wrote {}", out.join("deploy-aws.sh").display()));

    // Terraform module placeholder
    let tf_content = r#"# Terraform module for Oxidite Lambda function
# Update the image_uri after your first deploy.
resource "aws_lambda_function" "oxidite_fn" {
  function_name = "oxidite-serverless"
  package_type  = "Image"
  image_uri     = "${var.aws_account_id}.dkr.ecr.${var.aws_region}.amazonaws.com/oxidite-serverless:latest"
  role          = aws_iam_role.lambda_exec.arn
  timeout       = 30
  memory_size   = 512
  environment {
    variables = {
      OXIDITE_SERVERLESS = "1"
    }
  }
}

resource "aws_iam_role" "lambda_exec" {
  name = "oxidite-lambda-exec"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "basic_exec" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}
"#;
    fs::write(out.join("lambda.tf"), tf_content)?;
    output::info(&format!("  Wrote {}", out.join("lambda.tf").display()));

    output::success("AWS Lambda artifacts generated. See deploy/ for Dockerfile and scripts.");
    Ok(())
}

fn generate_docker(out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    output::step("Generating Docker deployment artifacts");

    let dockerfile = r#"FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/<YOUR_PROJECT> /app/server
EXPOSE 8080
CMD ["/app/server"]
"#;
    fs::write(out.join("Dockerfile"), dockerfile)?;
    output::info(&format!("  Wrote {}", out.join("Dockerfile").display()));

    let compose = r#"version: "3.9"
services:
  oxidite:
    build: ..
    ports:
      - "8080:8080"
    environment:
      - OXIDITE_ENV=production
      - SERVER_HOST=0.0.0.0
"#;
    fs::write(out.join("docker-compose.yml"), compose)?;
    output::info(&format!("  Wrote {}", out.join("docker-compose.yml").display()));

    output::success("Docker artifacts generated. See deploy/ for Dockerfile.");
    Ok(())
}

fn generate_cloudflare(out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    output::step("Generating Cloudflare Workers deployment artifacts");

    let wrangler = r#"name = "oxidite-function"
main = "src/main.rs"
compatibility_date = "2025-01-01"

[build]
command = "cargo build --release --target wasm32-wasi"
watch_dir = "src"

[[build.assets]]
pattern = "*.wasm"
"#;
    fs::write(out.join("wrangler.toml"), wrangler)?;
    output::info(&format!("  Wrote {}", out.join("wrangler.toml").display()));

    output::success("Cloudflare Workers config generated. See deploy/wrangler.toml.");
    Ok(())
}
