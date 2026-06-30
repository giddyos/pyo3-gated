# Release Checklist

1. Update `CHANGELOG.md` and confirm README examples use the release version.
2. Confirm `crates/pyo3-gated` and `crates/pyo3-gated-macros` have the same version.
3. Confirm `pyo3-gated` depends on the exact matching `pyo3-gated-macros` version.
4. Run:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo doc --workspace --all-features --no-deps
   cargo publish --dry-run -p pyo3-gated-macros
   ```

   `cargo package -p pyo3-gated` can only verify successfully after the matching
   `pyo3-gated-macros` version exists in the crates.io index.

5. Publish in dependency order:

   ```bash
   cargo publish -p pyo3-gated-macros
   # wait for the crates.io index, then verify the facade against the registry dependency
   cargo publish --dry-run -p pyo3-gated
   cargo publish -p pyo3-gated
   ```

Wait for the crates.io index to include `pyo3-gated-macros` before publishing `pyo3-gated`.

Recommended additional checks before a public release:

```bash
cargo deny check
cargo audit
cargo semver-checks check-release -p pyo3-gated-macros
cargo semver-checks check-release -p pyo3-gated
cargo machete
```
