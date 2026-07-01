# Contributing

Run the main validation script before opening a PR:

```bash
scripts/check.sh
```

If your change affects stub generation, also run:

```bash
scripts/stub-check.sh
```

UI tests use trybuild snapshots under `crates/pyo3-gated/tests/ui`. When a diagnostic intentionally changes, rerun the relevant test with `TRYBUILD=overwrite` and review the updated `.stderr` file before committing it.
