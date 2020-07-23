#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-accept-enum.rs");
    t.compile_fail("tests/02-reject-struct.rs");
    t.compile_fail("tests/03-reject-no-code.rs");
}
