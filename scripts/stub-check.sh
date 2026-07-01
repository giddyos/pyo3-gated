#!/usr/bin/env bash
set -euo pipefail

"${PYTHON:-${PYO3_PYTHON:-python3}}" - <<'PY'
import sys

if sys.version_info < (3, 10):
    raise SystemExit("pyo3-stub-gen checks require Python 3.10 or newer")
PY

cargo run -p stub-user --bin stub_gen --features stub-gen
cargo run -p pyo3-gated-color-module --bin stub_gen --features stub-gen
diff -u tests/expected-stubs/stub_user.pyi tests/crates/stub-user/stub_user.pyi
diff -u tests/expected-stubs/color_module.pyi examples/color-module/color_module.pyi
