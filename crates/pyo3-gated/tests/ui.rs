#[test]
fn ui_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/conflicting_sentinels.rs");
}
