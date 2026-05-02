#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo &>/dev/null; then
    echo "Error: cargo is not available. Please install Rust from https://rustup.rs" >&2
    exit 1
fi

script_dir="$(dirname "$0")"
cd "$script_dir"

cargo build --release
./target/release/sync install

echo "skill-sync installed successfully."
