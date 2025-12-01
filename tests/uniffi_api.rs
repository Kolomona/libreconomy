// TDD: Failing tests for uniffi FFI API
#[test]
fn test_libreconomy_version_uniffi() {
    // This should call the uniffi-generated Rust function
    let version = libreconomy::libreconomy_version();
    assert_eq!(version, "0.0.1"); // Expecting crate version
}

#[test]
fn test_get_agent_count_uniffi() {
    // This should call the uniffi-generated Rust function
    let count = libreconomy::get_agent_count();
    assert_eq!(count, 0); // Expecting 0 agents in default state
}
