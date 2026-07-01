# Maturin

The example module can be built as a wheel:

```bash
cd examples/color-module
maturin build --out ../../dist
python -m pip install ../../dist/*.whl
```

The example `pyproject.toml` enables the `python-extension` feature. In this repo that feature enables the downstream `python` feature, while `maturin` handles PyO3 extension-module build behavior.

For direct Cargo extension builds outside `maturin`, set `PYO3_BUILD_EXTENSION_MODULE=1`. Keep `pyo3-gated/extension-module` and `pyo3-gated/generate-import-lib` only for compatibility with older workflows; `generate-import-lib` is deprecated upstream in PyO3 0.29.

For local editable development, `maturin develop` requires an active virtual environment or conda environment.
