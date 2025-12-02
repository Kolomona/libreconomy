# Testing Guide

## Running Tests

- All tests: `cargo test`
- Unit tests only: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- Doc tests: `cargo test --doc`
- Specific test: `cargo test test_name`

## Running Benchmarks

- All benchmarks: `cargo bench`
- Specific benchmark: `cargo bench benchmark_name`

## Test Organization

- Unit tests: In same file as code using `#[cfg(test)]`
- Integration tests: In `tests/` directory
- Benchmarks: In `benches/` directory
- FFI completeness tests: `tests/ffi_completeness.rs` (special integration test)

## FFI/Bridge Completeness Testing

The `ffi_completeness` test suite ensures API consistency across all layers:

### What It Tests

1. **FFI Export Verification** (`test_all_expected_functions_exported_in_ffi_module`)
   - Verifies that all expected public Rust API functions have corresponding C FFI exports
   - Checks for both `pub extern "C"` and `pub unsafe extern "C"` function signatures
   - Fails if any public function is missing from the FFI layer

2. **Header Generation Validation** (`test_all_required_functions_in_generated_header`)
   - Ensures all FFI functions appear in the generated `libreconomy.h` header
   - Validates that cbindgen correctly picks up `#[no_mangle]` exports
   - Requires running `cargo build` first to generate the header

3. **Public API Coverage** (`test_ffi_functions_match_rust_public_api`)
   - Scans `src/agent/creation.rs` for public functions
   - Verifies each has a corresponding FFI export mapping
   - Prevents forgetting to export new public functions

4. **Bridge Completeness** (`test_bridge_cpp_uses_all_ffi_functions`)
   - Validates that all FFI functions are used in `libreconomy_bridge.cpp`
   - Ensures the Godot bridge exposes the complete API
   - Skips if bridge hasn't been generated yet (not a hard failure)

5. **No Orphaned Exports** (`test_no_extra_ffi_exports`)
   - Detects FFI exports that aren't documented in the test's expected mapping
   - Prevents accumulation of unused/forgotten exports
   - Maintains clean FFI surface

### When Tests Run

- **Manually**: `cargo test --test ffi_completeness`
- **Automatically**: Part of `scripts/release.sh` after `cargo build --release`
- **CI/CD**: Should be included in continuous integration pipelines

### Why This Matters

Without these tests, it's easy to:
- Add a new Rust function but forget to export it to FFI
- Export to FFI but forget to add it to the Godot bridge
- Leave orphaned FFI functions after refactoring
- Ship releases with incomplete API coverage

The tests catch these issues at build time, before artifacts are distributed.

### Updating the Tests

When adding new public API functions:

1. Add the Rust function (e.g., in `src/agent/creation.rs`)
2. Add corresponding FFI export in `src/ffi/mod.rs`
3. Update `EXPECTED_API_EXPORTS` in `tests/ffi_completeness.rs`
4. Run tests: `cargo test --test ffi_completeness`
5. Update the Godot bridge if needed (usually automatic via `release.sh`)

### Example Output

```bash
$ cargo test --test ffi_completeness

running 5 tests
test test_all_expected_functions_exported_in_ffi_module ... ok
test test_all_required_functions_in_generated_header ... ok
test test_ffi_functions_match_rust_public_api ... ok
test test_bridge_cpp_uses_all_ffi_functions ... ok
test test_no_extra_ffi_exports ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

## TDD Workflow

1. Write a failing test
2. Implement minimum code to pass
3. Refactor while keeping tests green
4. Add property tests for edge cases

## Tools

- `pretty_assertions`: Better assertion failure messages
- `proptest`: Property-based testing for edge cases
- `criterion`: Statistical benchmarking
