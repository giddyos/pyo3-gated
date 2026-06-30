# Contributing

Before opening a pull request, run:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --all-features --no-deps
```

Expansion snapshots are checked by `cargo test` when a modern `cargo-expand` is available. Install it with:

```bash
cargo install cargo-expand --locked
```

Feature-related changes should include coverage for plain Rust builds, Python-enabled builds, and stub-generation builds when applicable.
