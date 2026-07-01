use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest_dir.join("../..");

    if candidate.join("Cargo.toml").exists() && candidate.join("crates").exists() {
        candidate
    } else {
        manifest_dir.to_path_buf()
    }
}

fn configure_cargo_command(cmd: &mut Command) {
    if std::env::var_os("CARGO_TARGET_DIR").is_none() {
        cmd.env("CARGO_TARGET_DIR", workspace_root().join("target"));
    }
}

fn cargo_check(package: &str, features: &[&str]) {
    let mut cmd = Command::new("cargo");
    configure_cargo_command(&mut cmd);
    cmd.arg("check").arg("-p").arg(package);

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let status = cmd.status().expect("failed to run cargo check");
    assert!(status.success(), "cargo check failed for {package}");
}

fn cargo_check_manifest(manifest_path: &str, features: &[&str]) {
    let mut cmd = Command::new("cargo");
    configure_cargo_command(&mut cmd);
    cmd.arg("check").arg("--manifest-path").arg(manifest_path);

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let status = cmd.status().expect("failed to run cargo check");
    assert!(status.success(), "cargo check failed for {manifest_path}");
}

fn fixture_manifest(relative_path: &str) -> String {
    if Path::new(relative_path).exists() {
        return relative_path.to_string();
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative_path)
        .display()
        .to_string()
}

fn cargo_check_manifest_fails(manifest_path: &str, features: &[&str], expected: &str) {
    let mut cmd = Command::new("cargo");
    configure_cargo_command(&mut cmd);
    cmd.arg("check").arg("--manifest-path").arg(manifest_path);

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let output = cmd.output().expect("failed to run cargo check");
    assert!(
        !output.status.success(),
        "cargo check unexpectedly succeeded for {manifest_path}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "cargo check stderr did not contain expected text.\nexpected: {expected}\nstderr:\n{stderr}"
    );
}

#[test]
fn plain_user_compiles_without_pyo3() {
    cargo_check("plain-user", &[]);
}

#[test]
fn missing_pyo3_user_compiles_without_python_feature() {
    cargo_check_manifest(
        &fixture_manifest("tests/crates/missing-pyo3-user/Cargo.toml"),
        &[],
    );
}

#[test]
fn missing_pyo3_user_has_targeted_python_feature_error() {
    cargo_check_manifest_fails(
        &fixture_manifest("tests/crates/missing-pyo3-user/Cargo.toml"),
        &["python"],
        "pyo3-gated: enabling the Python feature requires a direct optional `pyo3` dependency",
    );
}

#[test]
fn python_user_compiles_without_direct_stub_gen_dep() {
    cargo_check("python-user", &["python"]);
}

#[test]
fn stub_user_compiles_without_direct_stub_gen_dep() {
    cargo_check("stub-user", &["stub-gen"]);
}

#[test]
fn renamed_dependency_user_compiles() {
    cargo_check("renamed-dep-user", &["python"]);
}

#[test]
fn renamed_pyo3_dependency_user_compiles() {
    cargo_check("renamed-pyo3-user", &["python"]);
}
