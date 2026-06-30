fn main() -> pyo3_gated::StubGenResult<()> {
    let stub = color_module::stub_info()?;
    stub.generate()?;
    Ok(())
}
