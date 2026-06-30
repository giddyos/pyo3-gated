use gated::{py_compat_methods, py_compat_struct};

gated::define_pyo3_gated_stub_info!(stub_info);

#[py_compat_struct]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
}

#[py_compat_methods]
impl Color {
    pub fn red(&self) -> u8 {
        self.r
    }
}
