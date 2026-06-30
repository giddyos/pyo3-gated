# pyo3-gated

[![CI](https://github.com/giddyos/pyo3-gated/actions/workflows/ci.yml/badge.svg)](https://github.com/giddyos/pyo3-gated/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/pyo3-gated.svg)](https://crates.io/crates/pyo3-gated)
[![docs.rs](https://docs.rs/pyo3-gated/badge.svg)](https://docs.rs/pyo3-gated)

Write Rust types once. Use them natively in Rust and optionally expose them to Python via PyO3 without duplicate definitions.

## Quick Start

```toml
[dependencies]
pyo3-gated = "^0.1"
pyo3       = { version = "0.28", optional = true }

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

```rust,ignore
use pyo3_gated::prelude::*;

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    #[pyo3(get, set)]
    pub g: u8,
    #[pyo3(get, set)]
    pub b: u8,
}

#[py_compat]
impl Color {
    #[py_attrs]
    #[new]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    #[py_only]
    pub fn __repr__(&self) -> String {
        format!("Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }

    #[rust_only]
    pub fn as_rgb_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

#[py_compat(pyclass_args(eq))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Red,
    Green,
    Blue,
}

#[py_compat(pyfunction_args(signature = (a, b = 0)))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pyo3_gated::define_py_module! {
    module color_module;
    classes: [Color, Palette];
    functions: [add];
}
```

`cargo build` compiles plain Rust without `pyo3`. `cargo build --features python` emits the PyO3 annotations without pulling stub-generation dependencies. `cargo run --bin stub_gen --features stub-gen` enables `pyo3-stub-gen` registration and stub output. Downstream crates own their direct `pyo3` dependency and features; `pyo3-stub-gen` is provided by `pyo3-gated` only behind `pyo3-gated/stub-gen`.

## Feature Model

`pyo3-gated` assumes a Cargo feature named `python` by default.

```toml
[dependencies]
pyo3-gated = "^0.1"
pyo3       = { version = "0.28", optional = true }

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

This keeps PyO3 feature choices, such as `extension-module`, `generate-import-lib`, `abi3`, `anyhow`, or other conversion features, under the downstream crate's control.

## Build Recipes

```bash
# Plain Rust build: no PyO3 dependency needed
cargo build

# Python-aware Rust build
cargo build --features python

# Extension-module build
cargo build --features python-extension

# Build and install the example module with maturin
maturin develop -m examples/color-module/pyproject.toml

# Generate stubs
cargo run --bin stub_gen --features stub-gen
```

## Macros

| Macro | Applies to |
|---|---|
| `py_compat` | dispatches over structs, enums, inherent impl blocks, and free functions |
| `py_compat_struct` | `struct` definitions |
| `py_compat_enum` | simple and complex `enum` definitions |
| `py_compat_methods` | inherent `impl` blocks |
| `py_compat_fn` | free functions |
| `define_py_module!` | cfg-gated PyO3 module registration |

Each macro emits two cfg-gated versions:

| Build | Output |
|---|---|
| `feature = "python"` | PyO3-annotated item |
| `feature = "stub-gen"` | stub-gen registration attributes |
| no `python` feature | plain Rust item with PyO3 and stub-gen attributes stripped |

## Method Sentinels

Inside `#[py_compat_methods]`, use these item-level marker attributes:

| Attribute | Effect |
|---|---|
| `#[py_only]` | method exists only in Python builds |
| `#[rust_only]` | method exists only in plain Rust builds and is not exposed to Python |
| `#[py_attrs]` | method exists in both builds, but Python-specific attributes are stripped in plain builds |

Combining `#[py_only]`, `#[rust_only]`, and `#[py_attrs]` on the same item is a compile error.

## Macro Arguments

| Argument | Values | Default | Purpose |
|---|---|---|---|
| `feature` | `"feature-name"` | `"python"` | Which Cargo feature enables the Python build |
| `stub_gen` | `false`, `true`, or `"feature-name"` | `"stub-gen"` | Controls automatic stub-registration derive emission |
| `pyclass_args` | token tree | none | Forwarded into `#[pyclass(...)]` |
| `pyfunction_args` | token tree | none | Forwarded into `#[pyfunction(...)]` |
| `pyo3_crate` | Rust path string | resolved from Cargo metadata | Override PyO3 crate path, for renamed or re-exported PyO3 |
| `py_only` | flag | false | Free function exists only in Python builds |

Stub registration is enabled by default under a Cargo feature named `stub-gen`. Disable it for one item when needed:

```rust,ignore
#[py_compat_struct(stub_gen = false)]
pub struct InternalOnly {
    pub raw: Vec<u8>,
}
```

Use a custom Python feature name:

```rust,ignore
#[py_compat_struct(feature = "pyo3", stub_gen = "stubs")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

Forward PyO3 class options:

```rust,ignore
#[py_compat_struct(pyclass_args(module = "palette", get_all))]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

Forward PyO3 function options:

```rust,ignore
#[py_compat_fn(pyfunction_args(name = "add_numbers", signature = (a, b = 0)))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Use a Python-only free function when its signature uses PyO3 types:

```rust,ignore
#[py_compat_fn(py_only)]
pub fn inspect_py_object(obj: pyo3::Bound<'_, pyo3::types::PyAny>) -> pyo3::PyResult<String> {
    Ok(format!("{obj:?}"))
}
```

Register a module without handwritten cfg boilerplate:

```rust,ignore
pyo3_gated::define_py_module! {
    module color_module;
    classes: [Color, Palette];
    functions: [add];
}
```

For unusual dependency layouts, override the PyO3 path:

```rust,ignore
#[py_compat_struct(pyo3_crate = "::py")]
pub struct Color {
    pub r: u8,
}
```

## Stub Generation

The macros choose the correct `pyo3-stub-gen` derive automatically:

| Item | Stub derive |
|---|---|
| struct | `gen_stub_pyclass` |
| simple enum | `gen_stub_pyclass_enum` |
| complex enum | `gen_stub_pyclass_complex_enum` |
| impl block | `gen_stub_pymethods` |
| free function | `gen_stub_pyfunction` |

Define the gatherer once in your library:

```rust,ignore
pyo3_gated::define_pyo3_gated_stub_info!(stub_info);
```

Or use the re-exported upstream macro:

```rust,ignore
#[cfg(feature = "stub-gen")]
pyo3_gated::define_stub_info_gatherer!(stub_info);
```

Then gate your stub-generation binary with `stub-gen`:

```toml
[[bin]]
name = "stub_gen"
path = "src/bin/stub_gen.rs"
required-features = ["stub-gen"]
```

```rust,ignore
fn main() -> pyo3_gated::StubGenResult<()> {
    let stub = your_crate::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

Advanced stub-generation APIs are available under `pyo3_gated::stub_gen` when `stub-gen` is enabled.

## Troubleshooting

`cannot find crate pyo3`: your downstream `python` feature likely enabled `pyo3-gated/python` but not your direct PyO3 dependency. Use `python = ["dep:pyo3", "pyo3-gated/python"]`.

`StubGenResult not found`: build the stub binary with the downstream `stub-gen` feature and wire that feature to `pyo3-gated/stub-gen`. Stub binaries should usually declare `required-features = ["stub-gen"]`.

Python-specific types fail in plain Rust builds: `#[py_attrs]` strips attributes, not Rust types. Methods taking `Python<'_>`, `Bound<'_, PyAny>`, `PyResult<T>`, or other PyO3 types should be `#[py_only]` or manually cfg-gated.

Missing `.pyi` content: confirm the item did not use `stub_gen = false`, the stub binary is run with `--features stub-gen`, and `pyclass_args(module = "...")` matches the module layout you expect.

## Migration From Raw PyO3

Before:

```rust,ignore
#[cfg_attr(feature = "python", pyclass)]
pub struct Color {
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub r: u8,
}

#[cfg_attr(feature = "python", pymethods)]
impl Color {
    #[new]
    pub fn new(r: u8) -> Self {
        Self { r }
    }
}
```

After:

```rust,ignore
#[py_compat_struct]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
}

#[py_compat_methods]
impl Color {
    #[py_attrs]
    #[new]
    pub fn new(r: u8) -> Self {
        Self { r }
    }
}
```

## Rust-Only Crates

Rust-only users only need:

```toml
[dependencies]
pyo3-gated = "^0.1"
```

PyO3 field, variant, and function attributes are stripped from the plain build, so no direct `pyo3` dependency is required unless the Python feature is enabled.

## Current Limitations

- `#[py_compat_methods]` supports inherent `impl Type { ... }` blocks, not trait impls.
- Users should not manually add `#[pyclass]`, `#[pymethods]`, `#[pyfunction]`, or `#[pymodule]` to items managed by these macros; the macros add them.
- Python builds still require a direct `pyo3` dependency because downstream crates own PyO3 feature selection.
- Python-specific argument and return types still require cfg control for plain Rust builds.
- `pyo3-gated` does not choose `abi3`, `extension-module`, `auto-initialize`, or conversion features for you.
- Generic PyO3 classes and complex enums remain subject to PyO3 and `pyo3-stub-gen` limitations.
- Stub generation requires project metadata such as `pyproject.toml` when generating package-oriented stubs.

## License

MIT
