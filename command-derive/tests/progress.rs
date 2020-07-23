#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-accept-enum.rs");
    t.compile_fail("tests/02-reject-struct.rs");
    t.compile_fail("tests/03-reject-no-code.rs");
    t.compile_fail("tests/04-fromstriter-reject-enum.rs");
    t.pass("tests/05-accept-fromstriter-both-named-and-unnamed.rs");
    t.compile_fail("tests/06-reject-wrong-code-lit.rs");
    t.pass("tests/07-flatten.rs");
}
