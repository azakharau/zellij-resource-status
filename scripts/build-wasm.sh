#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-wasip1/release"
OUT_WASM="$ROOT_DIR/target/zellij-resource-status.wasm"

rustup target add wasm32-wasip1 >/dev/null
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release --target wasm32-wasip1
cp "$TARGET_DIR/zellij-resource-status.wasm" "$OUT_WASM"
printf '%s\n' "$OUT_WASM"
