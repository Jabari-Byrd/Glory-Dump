#!/bin/bash
# Glory Dump IDL Build Fix Script
# This script implements the workaround for Anchor 0.30.x IDL build issues

set -euo pipefail

echo "🔧 Glory Dump IDL Build Fix"
echo "=============================="

# Ensure we're in the workspace root
cd "$(dirname "$0")"

echo "📍 Working directory: $(pwd)"

# Step 1: Ensure proc-macro2 is pinned to 1.0.94
echo "📌 Pinning proc-macro2 to version 1.0.94..."
/Users/Jabari/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo update -p proc-macro2 --precise 1.0.94

# Step 2: Try to build with current Anchor CLI
echo "🔨 Building with current Anchor CLI..."
if anchor build; then
    echo "✅ Build successful with current Anchor CLI"
else
    echo "❌ Build failed with current Anchor CLI, trying manual approach..."
    
    # Step 3: Manual SBF build
    echo "🔨 Building SBF manually..."
    cd programs/glory-dump-game
    RUSTUP_TOOLCHAIN=stable /Users/Jabari/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo build-bpf \
        --manifest-path Cargo.toml \
        --bpf-out-dir ../../target/deploy
    cd ../..
fi

# Step 4: Build IDL with correct nightly
echo "🌙 Building IDL with nightly-2025-04-01..."
export RUSTUP_TOOLCHAIN=nightly-2025-04-01

# Create IDL output directory
mkdir -p target/idl
mkdir -p target/types

# Try to build IDL - if the current CLI doesn't support it, we'll do it manually
if command -v anchor >/dev/null 2>&1; then
    echo "📋 Attempting IDL build with Anchor CLI..."
    if anchor idl build -o target/idl/glory_dump_game.json; then
        echo "✅ IDL built successfully"
    else
        echo "⚠️  IDL build failed with current Anchor CLI"
        echo "💡 You may need to upgrade to Anchor CLI 0.30.1+ for proper IDL support"
        echo "    Or implement manual IDL extraction"
    fi
else
    echo "❌ Anchor CLI not found"
fi

echo ""
echo "🎯 Summary:"
echo "- proc-macro2 pinned to 1.0.94 ✅"
echo "- SBF build attempted ✅"
echo "- IDL build attempted with nightly-2025-04-01 ✅"
echo ""
echo "📁 Check these directories for outputs:"
echo "- target/deploy/ for .so files"
echo "- target/idl/ for IDL JSON"
echo "- target/types/ for TypeScript types"
