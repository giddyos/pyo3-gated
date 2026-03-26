# pyo3-gated

Write your Rust types once. Use them natively in Rust and expose them to Python via PyO3. No duplicate definitions.

## Quick start

```toml
# Cargo.toml
[dependencies]
pyo3-gated = "^0.1"
pyo3         = { version = "0.28", optional = true }
pyo3-stub-gen = { version = "0.6", optional = true }

[features]
default          = []
python           = ["dep:pyo3", "dep:pyo3-stub-gen"]
python-extension = ["python", "pyo3/extension-module", "pyo3/generate-import-lib"]
```

```rust
use pyo3_gated::{py_compat_struct, py_compat_enum, py_compat_methods, py_compat_fn};

#[py_compat_struct(stub_gen = true)]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[py_compat_enum(pyclass_args(skip_from_py_object))]
#[derive(Clone, Copy, PartialEq)]
pub enum Palette { Red, Green, Blue }

#[py_compat_methods(stub_gen = true)]
impl Color {

    // Available in Rust and treated as a constructor in Python.
    #[py_attrs]
    #[new]
    pub fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }

    // Available in Rust and Python.
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    // `#[getter]` kept in Python builds, stripped in regular builds.
    #[py_attrs]
    #[getter]
    pub fn g(&self) -> u8 { self.g }

    // Only available in Python, never present in regular builds.
    #[py_only]
    pub fn __repr__(&self) -> String {
        format!("Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }
}

#[py_compat_fn(stub_gen = true)]
#[pyo3(signature = (a, b=0))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// When a `#[py_compat_fn]` returns a `Result<T, E>`, the error type `E` must be
// convertible to a Python error when building with the Python feature (i.e. it
// must implement a conversion into `PyErr`). For convenience and to keep the
// function usable from both Rust and Python without extra boilerplate, the
// recommended approach is to return `anyhow::Result<T>` and enable pyo3's
// `anyhow` feature in your `Cargo.toml` so `anyhow::Error` converts to `PyErr`.
#[py_compat_fn(stub_gen = true)]
pub fn divide(a: i32, b: i32) -> anyhow::Result<i32> {
    if b == 0 {
        // you could also use anyhow::bail!("error message");
        return Err(anyhow::anyhow!("Division by zero is not allowed"));
    }

    Ok(a / b)
}
```

`cargo build` compiles plain Rust. `cargo build --features python` generates a fully annotated PyO3 extension. That's it.

---

## Why this crate exists

When a type lives in both your Rust API and your Python bindings, the usual approach starts small and gets painful fast:

- A plain Rust `struct` or `enum` for your internal code.
- A separate `#[pyclass]` version for Python.
- Duplicate `impl` blocks, or heavy `cfg(feature = "python")` branching.
- Constant drift between the Rust-native and Python-exposed versions.

That duplication has real costs:

- Fields get added in one place and forgotten in the other.
- Methods diverge.
- Python-only items like `#[new]` or `__repr__` leak into code that should stay Rust-only.
- Your non-Python builds still have to tiptoe around PyO3-specific code paths.

`pyo3-gated` fixes that by generating two cfg-gated views from one definition:

- A **Python build** with `#[pyclass]` / `#[pymethods]`.
- A **plain Rust build** with PyO3 attributes stripped out completely.

The result is simple: write your type once, keep your logic in one place, and compile the right shape for the target you're building.

## What you get

### Single-source definitions
Use one `struct`, `enum`, or `impl` block instead of parallel Rust and PyO3 versions.

### Zero PyO3 dependency in plain builds
When your Python feature is off, the expanded plain version removes PyO3 attributes so the crate can compile without PyO3.

### Python-only items where they belong
Mark constructors, protocol methods, and similar items with `#[py_only]` so they exist only in Python-enabled builds.

### Shared methods without annotation noise
Use `#[py_attrs]` for items that should exist in both builds, but keep Python-specific attributes only in the Python version.

### Automatic `.pyi` registration
When enabled, the macros emit the matching `pyo3-stub-gen` derive automatically, including the correct enum variant for simple vs. complex enums.

### Feature-driven integration
The crate is designed around one obvious feature flag, so Rust-only consumers and Python-extension builds can coexist cleanly.

## Before and after

### Before: duplicated types

```rust
#[cfg(feature = "python")]
#[pyclass]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[cfg(not(feature = "python"))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[cfg(feature = "python")]
#[pymethods]
impl Color {
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    #[new]
    pub fn py_new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[cfg(not(feature = "python"))]
impl Color {
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}
```

This works, but it does not scale. Every change has to be made twice, and every impl block becomes a maintenance trap.

### After: one definition

```rust
use pyo3_gated::{py_compat_methods, py_compat_struct};

#[py_compat_struct]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[py_compat_methods]
impl Color {
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    #[py_only]
    #[new]
    pub fn py_new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
```

Same type. Same logic. One place to evolve.

## How it works

pyo3-gated provides four attribute macros:

| Macro | Applies to |
|---|---|
| [`py_compat_struct`] | `struct` definitions |
| [`py_compat_enum`] | `enum` definitions (simple and complex) |
| [`py_compat_methods`] | `impl` blocks |
| [`py_compat_fn`] | free function definitions |

`py_compat_fn` exposes a free function to Python via PyO3 while emitting
an identical plain-Rust version with PyO3 attributes stripped for non-Python
builds. This keeps a single implementation that is callable from Rust and,
when the Python feature is enabled, registered as a `#[pyfunction]`.

Example:

```rust
#[py_compat_fn]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

// With a PyO3 signature (kept in Python build, stripped in plain Rust):
#[py_compat_fn]
#[pyo3(signature = (a, b=0))]
pub fn add_with_default(a: u32, b: u32) -> u32 {
    a + b
}
```

### Error handling for `py_compat_fn`

Cargo.toml snippet:

```toml
[dependencies]
pyo3 = { version = "0.28", optional = true, features = ["anyhow"] }
anyhow = "1.0"

[features]
python = ["dep:pyo3", "dep:pyo3-stub-gen"]
```

Example using `anyhow::Result`:

```rust
#[py_compat_fn]
pub fn try_div(a: i32, b: i32) -> anyhow::Result<i32> {
    if b == 0 {
        anyhow::bail!("division by zero");
    }
    Ok(a / b)
}
```

This pattern keeps the Rust API ergonomic (returning `anyhow::Result` in
Rust-only builds) while letting PyO3 raise a suitable `PyErr` in Python
builds.

Inside `#[py_compat_methods]`, you can use two sentinels:

| Attribute | Meaning |
|---|---|
| `#[py_only]` | Item exists only in Python builds |
| `#[py_attrs]` | Item exists in both builds, but its attributes are stripped in plain builds |

This gives you a nice split:

- Shared Rust/Python business logic stays shared.
- Python protocol glue stays Python-only.
- Getter/setter-style annotations can stay attached to the same method without polluting Rust-only builds.

## Feature model

By default, the crate assumes a feature named `python`.

That means a typical setup looks like this:

```toml
[features]
default          = []
python           = ["dep:pyo3", "dep:pyo3-stub-gen"]
python-extension = ["python", "pyo3/extension-module", "pyo3/generate-import-lib"]
stub-gen         = ["python"]

[dependencies]
pyo3          = { version = "0.28.0", optional = true }
pyo3-stub-gen = { version = "0.6", optional = true }

[[bin]]
name = "stub_gen"
path = "src/bin/stub_gen.rs"
required-features = ["stub-gen"]
```

This layout keeps the crate ergonomic:

- `cargo build` stays Rust-only.
- `cargo build --features python` enables PyO3 bindings and stub registration.
- `cargo build --features python-extension` is a natural extension-module build.
- `stub-gen` exists only to gate the stub binary, not your core library.

## Macro arguments

All three macros accept the same optional arguments:

| Argument | Values | Default | Purpose |
|---|---|---|---|
| `feature` | `"feature-name"` | `"python"` | Which Cargo feature enables the Python build |
| `stub_gen` | `false`, `true`, or `"feature-name"` | `"python"` | Controls automatic stub-registration derive emission |
| `pyclass_args` | token tree | none | Forwarded into `#[pyclass(...)]` |

### Custom Python feature

```rust
#[py_compat_struct(feature = "pyo3")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

### Disable stub generation for one type

```rust
#[py_compat_struct(stub_gen = false)]
pub struct InternalOnly {
    pub raw: Vec<u8>,
}
```

### Forward `#[pyclass(...)]` options

```rust
#[py_compat_struct(pyclass_args(module = "my_pkg", get_all))]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

This keeps the macro lightweight: it handles the cfg split, while you still retain control over PyO3 class configuration.

## Stub generation

When stub generation is enabled, `pyo3-gated` emits the matching `pyo3-stub-gen` derive for you automatically.

You do not need to manually pick the enum stub kind:

- `struct` → `gen_stub_pyclass`
- simple enum → `gen_stub_pyclass_enum`
- complex enum → `gen_stub_pyclass_complex_enum`
- methods → `gen_stub_pymethods`

You still need to define the stub info gatherer once in your library crate:

```rust
// lib.rs
#[cfg(feature = "python")]
pyo3_stub_gen::define_stub_info_gatherer!(stub_info);
```

And then your stub-generation binary can gather and write the `.pyi` output in the usual way for your project.

## A realistic pattern

A good mental model is:

- Use normal Rust types and methods for your real domain logic.
- Add PyO3 field or method attributes where Python exposure matters.
- Mark Python-only glue with `#[py_only]`.
- Let `pyo3-gated` generate the split for you.

For example:

```rust
use pyo3_gated::{py_compat_methods, py_compat_struct};

#[py_compat_struct(pyclass_args(module = "palette"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    #[pyo3(get, set)]
    pub g: u8,
    #[pyo3(get, set)]
    pub b: u8,
}

#[py_compat_methods]
impl Color {
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn is_grayscale(&self) -> bool {
        self.r == self.g && self.g == self.b
    }

    #[py_attrs]
    #[new]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[py_only]
    pub fn __repr__(&self) -> String {
        format!("Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }
}
```

That gives you:

- A clean Rust type in normal builds.
- A Python class in Python-enabled builds.
- No duplicated methods.
- No parallel maintenance burden.

## When this crate is a good fit

`pyo3-gated` is especially useful when:

- Your crate is primarily a Rust library, but optionally exposes Python bindings.
- You publish both a Rust API and a Python extension from the same codebase.
- You want to keep domain types free of hand-written cfg duplication.
- You use `pyo3-stub-gen` and want stub registration to happen automatically.
- You care about keeping non-Python builds lean and easy to compile.

It is less useful if your Python layer is intentionally a completely separate API surface from your Rust one. In that case, explicit wrapper types may still be the better design.

## Installation

```toml
[dependencies]
pyo3-gated = "0.1"
```

For optional Python bindings:

```toml
[dependencies]
pyo3 = { version = "0.28", optional = true }
pyo3-stub-gen = { version = "0.6", optional = true }

[features]
default = []
python = ["dep:pyo3", "dep:pyo3-stub-gen"]
```

## Design goals

This crate aims to be:

- **Minimal** — it should remove boilerplate, not impose a framework.
- **Predictable** — the expanded code should match what you would have written by hand.
- **Rust-first** — your plain build should remain a first-class path.
- **PyO3-friendly** — Python-specific annotations should still feel native where they belong.

The crate is best thought of as a code-generation convenience layer, not a replacement for understanding PyO3 itself.


 ### MIT
