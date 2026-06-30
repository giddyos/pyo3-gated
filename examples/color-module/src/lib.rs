use pyo3_gated::prelude::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pyo3_gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat_struct(pyclass_args(module = "color_module", skip_from_py_object))]
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
    #[py_attrs]
    #[new]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    #[py_only]
    pub fn __repr__(&self) -> String {
        format!("Color(r={}, g={}, b={})", self.r, self.g, self.b)
    }
}

#[py_compat_enum(pyclass_args(skip_from_py_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    Red,
    Green,
    Blue,
}

#[py_compat_fn]
#[pyo3(signature = (a, b = 0))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(feature = "python")]
#[pymodule]
fn color_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Color>()?;
    m.add_class::<Palette>()?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}
