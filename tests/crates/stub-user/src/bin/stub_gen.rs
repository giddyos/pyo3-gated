fn main() -> pyo3_gated::StubGenResult<()> {
    let stub = stub_user::stub_info()?;
    stub.generate()?;
    Ok(())
}
