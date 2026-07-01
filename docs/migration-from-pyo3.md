# Migration From PyO3

Replace repeated `cfg_attr` annotations with the item macro that matches the Rust item, or with the dispatcher macro:

```rust,ignore
#[py_compat]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
}

#[py_compat]
impl Color {
    #[py_attrs]
    #[new]
    pub fn new(r: u8) -> Self {
        Self { r }
    }
}
```

Use `#[py_attrs]` for methods that are available in Rust and Python but have Python-specific method attributes. Use `#[py_only]` for Python-only methods and `#[rust_only]` for Rust-only methods.

Use `define_py_module!` to replace handwritten cfg-gated `#[pymodule]` registration.

`pyo3-gated 0.1.x` targets PyO3 0.29. PyO3 0.29 no longer supports Python 3.7, and stub generation should run on Python 3.10+.

Avoid adding a direct `pyo3` dependency while migrating unless you are intentionally testing dependency override behavior. Import explicit PyO3 types through `pyo3_gated::pyo3`; if an upgrade resolves duplicate PyO3 versions, inspect it with `cargo tree -i pyo3`.

For extension modules, prefer `maturin` or set `PYO3_BUILD_EXTENSION_MODULE=1` for direct Cargo builds. The facade still exposes `extension-module` and `generate-import-lib`, but `generate-import-lib` is deprecated upstream and should not be used in new migrations.
