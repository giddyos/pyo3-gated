# Feature Model

`pyo3-gated` keeps normal Python builds separate from stub generation:

```toml
[features]
default = []
python = ["dep:pyo3", "pyo3-gated/python"]
stub-gen = ["python", "pyo3-gated/stub-gen"]
python-extension = [
    "python",
    "pyo3/extension-module",
    "pyo3/generate-import-lib",
]
```

The downstream crate owns its direct `pyo3` dependency and all PyO3 feature choices. `pyo3-gated/python` only tells the macros to emit PyO3 annotations. `pyo3-gated/stub-gen` enables `pyo3-stub-gen` integration.

Use `feature = "name"` on macros when your downstream Python feature is not named `python`. Use `stub_gen = "name"` when stub registration is controlled by a different feature.
