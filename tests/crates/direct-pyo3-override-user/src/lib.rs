use pyo3_gated::prelude::*;

#[py_compat_struct(pyclass_args(skip_from_py_object), pyo3_crate = "::py")]
#[derive(Clone)]
pub struct UsesExplicitRenamedPyO3 {
    #[pyo3(get, set)]
    pub value: u8,
}

#[py_compat_methods(pyo3_crate = "::py")]
impl UsesExplicitRenamedPyO3 {
    #[new]
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}
