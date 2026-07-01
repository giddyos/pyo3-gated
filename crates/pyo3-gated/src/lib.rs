#![doc = include_str!("../../../README.md")]

pub use pyo3_gated_macros::{
    define_py_module, py_compat, py_compat_enum, py_compat_fn, py_compat_methods, py_compat_struct,
};

#[doc(hidden)]
pub use pyo3_gated_macros::__pyo3_gated_stub_gen_alias;

#[cfg(feature = "stub-gen")]
pub use pyo3_stub_gen::Result as StubGenResult;

// Macro-private compatibility surface: pyo3-stub-gen derives currently emit
// absolute `::pyo3_stub_gen::...` paths. The helper macro aliases this facade
// crate as `pyo3_stub_gen`, so these root items must exist but are not part of
// the documented stable API.
#[doc(hidden)]
#[cfg(feature = "stub-gen")]
pub use pyo3_stub_gen::*;

#[cfg(feature = "stub-gen")]
pub mod stub_gen {
    pub use pyo3_stub_gen::*;
}

pub mod prelude {
    pub use crate::{
        define_py_module, py_compat, py_compat_enum, py_compat_fn, py_compat_methods,
        py_compat_struct,
    };

    #[cfg(feature = "stub-gen")]
    pub use crate::StubGenResult;
}

#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "stub-gen")]
    pub use pyo3_stub_gen;
}

#[macro_export]
macro_rules! define_pyo3_gated_stub_info {
    ($name:ident) => {
        #[cfg(feature = "stub-gen")]
        $crate::__pyo3_gated_stub_gen_alias!();

        #[cfg(feature = "stub-gen")]
        $crate::__private::pyo3_stub_gen::define_stub_info_gatherer!($name);
    };
}

#[macro_export]
macro_rules! define_stub_info_gatherer {
    ($name:ident) => {
        $crate::define_pyo3_gated_stub_info!($name);
    };
}
