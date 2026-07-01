#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo check --workspace --all-targets
cargo check -p facade-pyo3-user
cargo check -p facade-pyo3-user --features python
cargo check -p facade-pyo3-user --features stub-gen
cargo check -p facade-pyo3-user --features anyhow
cargo check -p facade-pyo3-user --features abi3-py39
cargo check -p renamed-facade-user --features python
cargo check -p direct-pyo3-override-user --features python
cargo tree --workspace --features stub-gen -i pyo3
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo doc -p pyo3-gated --features full,stub-gen --no-deps
