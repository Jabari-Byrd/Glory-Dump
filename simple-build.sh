#!/bin/bash
# Simplified Glory Dump Build Script
# Focus on building what we can with the current toolchain

set -euo pipefail

echo "🔧 Glory Dump Simplified Build"
echo "==============================="

# Ensure we're in the workspace root
cd "$(dirname "$0")"
echo "📍 Working directory: $(pwd)"

# Check if proc-macro2 is already pinned
echo "📋 Checking proc-macro2 version..."
if grep -q "proc-macro2.*1\.0\.94" Cargo.lock; then
    echo "✅ proc-macro2 is pinned to 1.0.94"
else
    echo "📌 Pinning proc-macro2 to version 1.0.94..."
    /Users/Jabari/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo update -p proc-macro2 --precise 1.0.94
fi

# Try to build with anchor, but handle the failure gracefully
echo "🔨 Attempting build with Anchor CLI..."
if anchor build 2>/dev/null; then
    echo "✅ Anchor build succeeded!"
    echo "📁 Check target/deploy/ for .so files"
    echo "📁 Check target/idl/ for IDL files"
    echo "📁 Check target/types/ for TypeScript types"
else
    echo "⚠️  Anchor build failed (expected due to toolchain issues)"
    echo ""
    echo "🎯 Manual workaround required:"
    echo "1. ✅ proc-macro2 is pinned to 1.0.94"
    echo "2. ⚠️  Need newer Anchor CLI (0.30.1+) for proper IDL support"
    echo "3. ⚠️  Current Anchor CLI (0.29.0) has workspace resolution issues"
    echo ""
    echo "📋 Recommended next steps:"
    echo "   - Install Anchor CLI 0.30.1+ using AVM or cargo"
    echo "   - Or use manual SBF build with newer toolchain"
    echo "   - Then run: RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build"
fi

echo ""
echo "🔧 Current Status:"
echo "- proc-macro2 pinned: ✅"
echo "- Solana toolchain: ⚠️  (version conflicts)"
echo "- Anchor CLI: ⚠️  (v0.29.0, needs v0.30.1+)"
echo "- IDL workaround ready: ✅"
