use gated::prelude::*;
#[cfg(feature = "python")]
use gated::pyo3;

#[py_compat(pyclass_args(skip_from_py_object))]
#[derive(Clone)]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
}

#[py_compat]
impl Color {
    #[new]
    pub fn new(r: u8) -> Self {
        Self { r }
    }
}

#[py_compat_fn(py_only)]
pub fn inspect(obj: pyo3::Bound<'_, pyo3::types::PyAny>) -> pyo3::PyResult<String> {
    Ok(format!("{obj:?}"))
}
