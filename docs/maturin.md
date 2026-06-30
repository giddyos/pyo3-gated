# Maturin

The example module can be built as a wheel:

```bash
cd examples/color-module
maturin build --out ../../dist
python -m pip install ../../dist/*.whl
```

The example `pyproject.toml` enables the `python-extension` feature. That feature should include the downstream `python` feature plus PyO3 extension-module options.

For local editable development, `maturin develop` requires an active virtual environment or conda environment.
