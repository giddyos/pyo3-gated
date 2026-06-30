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
