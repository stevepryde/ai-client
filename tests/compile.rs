#[test]
fn responses_builder_capabilities_are_compile_checked() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass_*.rs");
    tests.compile_fail("tests/ui/fail_*.rs");
}

#[cfg(feature = "openai-compatible")]
#[test]
fn compatible_builder_dialect_and_capabilities_are_compile_checked() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/compatible/pass_*.rs");
    tests.compile_fail("tests/ui/compatible/fail_*.rs");
}
