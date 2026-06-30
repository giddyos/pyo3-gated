# Release Checklist

1. Update `CHANGELOG.md` and confirm README examples use the release version.
2. Confirm `crates/pyo3-gated` and `crates/pyo3-gated-macros` have the same version.
3. Confirm `pyo3-gated` depends on the exact matching `pyo3-gated-macros` version.
4. Run:

   ```bash
   cargo fmt --all -- --check
   cargo test --workspace
   cargo publish --dry-run -p pyo3-gated-macros
   ```

5. Publish in dependency order:

   ```bash
   cargo publish -p pyo3-gated-macros
   # wait for the crates.io index, then verify the facade against the registry dependency
   cargo publish --dry-run -p pyo3-gated
   cargo publish -p pyo3-gated
   ```

Wait for the crates.io index to include `pyo3-gated-macros` before publishing `pyo3-gated`.
