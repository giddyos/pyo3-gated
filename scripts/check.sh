#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo check --workspace --all-targets
cargo check -p facade-pyo3-user
cargo check -p facade-pyo3-user --features python
cargo check -p facade-pyo3-user --features stub-gen
cargo check -p facade-pyo3-user --features anyhow
cargo check -p facade-pyo3-user --features abi3-py39
PYO3_CROSS=1 PYO3_CROSS_PYTHON_VERSION=3.15 cargo check -p facade-pyo3-user --features abi3-py315
cargo check -p facade-pyo3-user --features abi3t-py315
cargo check -p facade-pyo3-user --features experimental-inspect
cargo check -p renamed-facade-user --features python
cargo check -p direct-pyo3-override-user --features python
cargo tree --workspace --features stub-gen -i pyo3
versions="$(cargo tree --workspace --features stub-gen -i pyo3 | sed -n 's/^pyo3 v\([^ ]*\).*/\1/p' | sort -u)"
count="$(printf '%s\n' "$versions" | sed '/^$/d' | wc -l | tr -d ' ')"
if [ "$count" != "1" ]; then
    printf 'expected exactly one pyo3 version, found:\n%s\n' "$versions"
    exit 1
fi
cargo test -p pyo3-gated --test feature_surface
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo doc -p pyo3-gated --features full,stub-gen --no-deps
