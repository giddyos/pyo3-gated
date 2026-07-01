#!/usr/bin/env bash
set -euo pipefail

PYTHON_BIN="${PYTHON:-${PYO3_PYTHON:-python3}}"

"$PYTHON_BIN" - <<'PY'
import sys

if sys.version_info < (3, 10):
    raise SystemExit("pyo3-stub-gen checks require Python 3.10 or newer")
PY

cargo run -p stub-user --bin stub_gen --features stub-gen
cargo run -p pyo3-gated-color-module --bin stub_gen --features stub-gen
diff -u tests/expected-stubs/stub_user.pyi tests/crates/stub-user/stub_user.pyi
diff -u tests/expected-stubs/color_module.pyi examples/color-module/color_module.pyi

dist_dir="$(mktemp -d)"
trap 'rm -rf "$dist_dir"' EXIT
(cd examples/color-module && maturin build --out "$dist_dir")
"$PYTHON_BIN" - "$dist_dir" <<'PY'
import sys
import zipfile
from pathlib import Path

dist = Path(sys.argv[1])
wheel = next(dist.glob("*.whl"))
with zipfile.ZipFile(wheel) as z:
    names = set(z.namelist())

assert any(name.endswith("color_module.pyi") for name in names), sorted(names)
PY
