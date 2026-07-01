use pyo3_gated::prelude::*;

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat(pyclass_args(skip_from_py_object), pyo3_crate = "::py")]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
}

#[py_compat(pyo3_crate = "::py")]
impl Color {
    #[py_attrs]
    #[new]
    pub fn new(r: u8) -> Self {
        Self { r }
    }

    pub fn red(&self) -> u8 {
        self.r
    }
}

#[py_compat(pyfunction_args(name = "add_numbers"), pyo3_crate = "::py")]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pyo3_gated::define_py_module! {
    pyo3_crate = "::py";
    module renamed_pyo3_user;
    classes: [Color];
    functions: [add];
}
