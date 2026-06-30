use pyo3_gated::prelude::*;

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat(pyclass_args(skip_from_py_object))]
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

    pub fn red(&self) -> u8 {
        self.r
    }
}

#[py_compat(pyfunction_args(name = "add_numbers"))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pyo3_gated::define_py_module! {
    module renamed_pyo3_user;
    classes: [Color];
    functions: [add];
}
