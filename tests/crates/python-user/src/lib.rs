use pyo3_gated::prelude::*;

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[allow(unused_macros)]
macro_rules! rust_only_macro_item {
    () => {
        pub const MACRO_VALUE: u8 = 7;
    };
}

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

    #[rust_only]
    pub fn into_inner(self) -> u8 {
        self.r
    }

    #[rust_only]
    pub const CHANNELS: u8 = 1;

    #[rust_only]
    rust_only_macro_item!();
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

#[py_compat_fn(pyfunction_args(name = "add_numbers", signature = (a, b = 0)))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[py_compat_fn(py_only)]
pub fn inspect_py_object(_obj: pyo3::Bound<'_, pyo3::types::PyAny>) -> pyo3::PyResult<String> {
    Ok("object".to_string())
}

pyo3_gated::define_py_module! {
    module python_user;
    doc: "Python-user fixture module.";
    classes: [Color, Palette];
    functions: [add, inspect_py_object];
    constants: [("VERSION", env!("CARGO_PKG_VERSION"))];
    init: |m| {
        let _ = m;
        Ok(())
    };
}
