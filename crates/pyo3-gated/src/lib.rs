//! Write Rust types once and optionally expose them to Python via PyO3.
//!
//! Normal builds do not require PyO3. Python-enabled builds require the
//! downstream crate to enable its own `pyo3` dependency and the
//! `pyo3-gated/python` feature.

pub use pyo3_gated_macros::{py_compat_enum, py_compat_fn, py_compat_methods, py_compat_struct};

#[doc(hidden)]
pub use pyo3_gated_macros::__pyo3_gated_stub_gen_alias;

#[cfg(feature = "python")]
pub use pyo3_stub_gen::*;

#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "python")]
    pub use pyo3_stub_gen;
}

#[macro_export]
macro_rules! define_pyo3_gated_stub_info {
    ($name:ident) => {
        #[cfg(feature = "python")]
        $crate::__pyo3_gated_stub_gen_alias!();

        #[cfg(feature = "python")]
        $crate::__private::pyo3_stub_gen::define_stub_info_gatherer!($name);
    };
}

#[macro_export]
macro_rules! define_stub_info_gatherer {
    ($name:ident) => {
        $crate::define_pyo3_gated_stub_info!($name);
    };
}
