use pyo3_gated::{py_compat_enum, py_compat_fn, py_compat_methods, py_compat_struct};

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat_struct(pyclass_args(module = "python_user", skip_from_py_object))]
#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn red(&self) -> u8 {
        self.r
    }
}

#[py_compat_enum(pyclass_args(skip_from_py_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    Red,
    Green,
}

#[py_compat_enum(pyclass_args(skip_from_py_object))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Swatch {
    Rgb { r: u8, g: u8, b: u8 },
    Named(String),
}

#[py_compat_fn]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
