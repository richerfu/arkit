#[test]
fn entry_rejects_invalid_contracts() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
