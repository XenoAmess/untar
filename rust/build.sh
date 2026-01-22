#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Building untar Rust binary..."

echo "Build static binary"
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
