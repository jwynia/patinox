#!/bin/bash
# Local CI validation script - runs the same checks as GitHub Actions

set -e  # Exit on any error

echo "🔍 Running local CI validation..."
echo

echo "✅ Step 1: Check formatting"
cargo fmt --check
echo "✅ Formatting OK"
echo

echo "✅ Step 2: Run clippy (with warnings as errors)"
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy OK"
echo

echo "✅ Step 3: Build project"
cargo build
echo "✅ Build OK"
echo

echo "✅ Step 4: Run tests"
cargo test
echo "✅ Tests OK"
echo

echo "✅ Step 5: Run doc tests"
cargo test --doc
echo "✅ Doc tests OK"
echo

echo "🎉 All local CI checks passed! Ready to push."