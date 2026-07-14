#[test]
fn i18n_rejects_invalid_declarations() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
