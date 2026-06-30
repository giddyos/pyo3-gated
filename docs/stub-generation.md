# Stub Generation

Enable stub generation with a dedicated feature:

```toml
stub-gen = ["python", "pyo3-gated/stub-gen"]

[[bin]]
name = "stub_gen"
path = "src/bin/stub_gen.rs"
required-features = ["stub-gen"]
```

Define the gatherer once:

```rust,ignore
pyo3_gated::define_pyo3_gated_stub_info!(stub_info);
```

Then generate stubs:

```rust,ignore
fn main() -> pyo3_gated::StubGenResult<()> {
    let stub = my_crate::stub_info()?;
    stub.generate()?;
    Ok(())
}
```

Use `stub_gen = false` for internal items that should not appear in `.pyi` output.
