use pyo3_gated::py_compat_fn;

#[py_compat_fn(stub_gen = false)]
#[gen_stub]
#[pyo3(signature = (a, b = 0))]
pub fn add(#[pyo3(from_py_with = "convert")] a: i32, b: i32) -> i32 {
    a + b
}

fn main() {}
