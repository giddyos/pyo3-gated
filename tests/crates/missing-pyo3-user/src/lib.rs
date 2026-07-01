use pyo3_gated::{py_compat_fn, py_compat_methods, py_compat_struct};

#[py_compat_struct]
pub struct Color {
    pub r: u8,
}

#[py_compat_methods]
impl Color {
    pub fn red(&self) -> u8 {
        self.r
    }
}

#[py_compat_fn]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
