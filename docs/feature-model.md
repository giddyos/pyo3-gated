# Feature Model

`pyo3-gated` keeps normal Python builds separate from stub generation:

```toml
[dependencies]
pyo3-gated = "^0.1"

[features]
default = []
python = ["pyo3-gated/python"]
stub-gen = ["python", "pyo3-gated/stub-gen"]
python-extension = [
    "python",
]
```

`pyo3-gated/python` enables `pyo3-gated`'s owned PyO3 dependency and tells the macros to emit PyO3 annotations against the facade re-export. `pyo3-gated/stub-gen` enables `pyo3-stub-gen` integration.

Use `maturin` for extension-module builds when possible. For direct Cargo extension builds, set `PYO3_BUILD_EXTENSION_MODULE=1`; `pyo3-gated/extension-module` remains available for older workflows. `pyo3-gated/generate-import-lib` is still exposed for compatibility, but PyO3 0.29 deprecates it and new projects should not enable it by default.

PyO3 pass-through features include ABI features such as `abi3-py315`, `abi3t`, and `abi3t-py315`. ABI-selection features are intentionally not included in `full`.

When code needs explicit PyO3 types, import them from the facade:

```rust,ignore
use pyo3_gated::pyo3;
```

Use `feature = "name"` on macros when your downstream Python feature is not named `python`. Use `stub_gen = "name"` when stub registration is controlled by a different feature.
