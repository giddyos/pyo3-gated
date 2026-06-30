use std::process::Command;

fn cargo_check(package: &str, features: &[&str]) {
    let mut cmd = Command::new("cargo");
    cmd.arg("check").arg("-p").arg(package);

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let status = cmd.status().expect("failed to run cargo check");
    assert!(status.success(), "cargo check failed for {package}");
}

#[test]
fn plain_user_compiles_without_pyo3() {
    cargo_check("plain-user", &[]);
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
