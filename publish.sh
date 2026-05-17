#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "=== Starting Oxidite Workspace Release Process (v2.2.0-beta) ==="

# Define publishing sequence in strict topological order
PUBLISH_ORDER=(
    "oxidite-utils"
    "oxidite-macros"
    "oxidite-config"
    "oxidite-db"
    "oxidite-core"
    "oxidite-testing"
    "oxidite-storage"
    "oxidite-security"
    "oxidite-plugin"
    "oxidite-auth"
    "oxidite-cache"
    "oxidite-graphql"
    "oxidite-mail"
    "oxidite-middleware"
    "oxidite-openapi"
    "oxidite-queue"
    "oxidite-realtime"
    "oxidite-template"
    "oxidite"
    "oxidite-cli"
)

TOTAL_CRATES=${#PUBLISH_ORDER[@]}
INTERVAL_SECS=30

for i in "${!PUBLISH_ORDER[@]}"; do
    CRATE="${PUBLISH_ORDER[$i]}"
    CRATE_NUM=$((i + 1))
    
    echo "--------------------------------------------------------"
    echo "[$CRATE_NUM/$TOTAL_CRATES] Publishing crate: $CRATE..."
    echo "--------------------------------------------------------"
    
    # Run publish command (using --allow-dirty for release tagging verification)
    cargo publish --manifest-path "$CRATE/Cargo.toml" --allow-dirty
    
    if [ $CRATE_NUM -lt $TOTAL_CRATES ]; then
        echo "Crate $CRATE published successfully. Waiting $INTERVAL_SECS seconds for crates.io index propagation..."
        sleep $INTERVAL_SECS
    fi
done

echo "=== All $TOTAL_CRATES crates successfully published to crates.io! ==="
