fn main() -> compile_test_utils::ui_test::Result<()> {
    // Run all tests in the tests/example_tests folder.
    // If we had more tests we could either call this function
    // on every single one or use test_multiple and past it an array
    // of paths.
    //
    // Don't forget that when running tests the working directory
    // is set to the crate root.
    compile_test_utils::test("tests/example_tests")
}
