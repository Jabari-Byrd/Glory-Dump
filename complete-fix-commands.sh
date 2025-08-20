#!/bin/bash
# Glory Dump IDL Build - Complete Fix Commands
# Run these commands to complete the Anchor IDL build fix

set -euo pipefail

echo "🎯 Glory Dump IDL Build - Final Fix Commands"
echo "============================================="

# Ensure we're in the workspace root
cd "$(dirname "$0")"
echo "📍 Working directory: $(pwd)"

echo ""
echo "✅ ALREADY COMPLETED:"
echo "- proc-macro2 pinned to 1.0.94"
echo "- nightly-2025-04-01 toolchain installed"
echo ""

echo "🔧 NEXT STEPS TO COMPLETE THE FIX:"
echo ""

echo "1️⃣ UPGRADE ANCHOR CLI (choose one option):"
echo ""
echo "   Option A: Install AVM and upgrade"
echo "   $ npm install -g @coral-xyz/anchor-cli  # may need sudo"
echo "   $ avm install 0.30.1"
echo "   $ avm use 0.30.1"
echo ""
echo "   Option B: Direct cargo install (recommended)"
echo "   $ cargo uninstall anchor-cli"
echo "   $ cargo install --git https://github.com/coral-xyz/anchor anchor-cli --tag v0.30.1"
echo ""

echo "2️⃣ AFTER ANCHOR CLI UPGRADE:"
echo ""
echo "   Build SBF without IDL:"
echo "   $ anchor build --no-idl"
echo ""
echo "   Build IDL with safe nightly:"
echo "   $ RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -o target/idl/glory_dump_game.json"
echo ""

echo "3️⃣ VERIFY SUCCESS:"
echo "   Check for these files:"
echo "   - target/deploy/glory_dump_game.so"
echo "   - target/idl/glory_dump_game.json"
echo "   - target/types/glory_dump_game.ts"
echo ""

echo "🚨 IF ANCHOR CLI UPGRADE FAILS:"
echo "   Try manual SBF build:"
echo "   $ cd programs/glory-dump-game"
echo "   $ rustup run stable cargo build-bpf --manifest-path Cargo.toml --bpf-out-dir ../../target/deploy"
echo ""

echo "📋 STATUS SUMMARY:"
echo "✅ Root cause fixed: proc-macro2 pinned to 1.0.94"
echo "✅ Safe nightly installed: nightly-2025-04-01"
echo "⚠️  Need: Anchor CLI 0.30.1+ for --no-idl flag"
echo "⚠️  Need: Proper toolchain setup"
echo ""

echo "🎯 TL;DR COMMANDS (after Anchor CLI upgrade):"
echo "cargo update -p proc-macro2 --precise 1.0.94  # ✅ DONE"
echo "anchor build --no-idl"
echo "RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build"
echo ""

echo "📚 For detailed troubleshooting, see: IDL-FIX-STATUS.md"
