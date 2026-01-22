#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Building untar Rust binary..."

# Build optimized binary
cargo build --release

echo "Binary size:"
ls -lh target/release/untar

echo ""
echo "To build static binary (requires musl target):"
echo "  rustup target add x86_64-unknown-linux-musl"
echo "  cargo build --release --target x86_64-unknown-linux-musl"
