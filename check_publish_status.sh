#!/bin/bash

# Script to check the publish status of all Oxidite crates
# This helps determine which crates have been published and which still need to be published

echo "Checking publish status of Oxidite crates..."

# Array of all Oxidite crates in publishing order
crates=(
    "oxidite-macros"
    "oxidite-utils" 
    "oxidite-security"
    "oxidite-core"
    "oxidite-config"
    "oxidite-db"
    "oxidite-cache"
    "oxidite-auth"
    "oxidite-template"
    "oxidite-mail"
    "oxidite-storage"
    "oxidite-queue"
    "oxidite-realtime"
    "oxidite-openapi"
    "oxidite-graphql"
    "oxidite-middleware"
    "oxidite-plugin"
    "oxidite-testing"
    "oxidite-cli"
    "oxidite"
)

echo "Checking each crate on crates.io..."
echo ""

for crate in "${crates[@]}"; do
    echo -n "Checking $crate: "
    
    # Check if crate exists on crates.io
    if curl -s "https://crates.io/api/v1/crates/$crate" | grep -q "crate"; then
        echo "✅ ALREADY PUBLISHED"
    else
        echo "⏳ NOT PUBLISHED"
    fi
done

echo ""
echo "To publish a crate, navigate to its directory and run: cargo publish"
echo "Remember to publish in dependency order!"