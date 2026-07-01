#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --all-features --no-deps
