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

## TDD Workflow

1. Write a failing test
2. Implement minimum code to pass
3. Refactor while keeping tests green
4. Add property tests for edge cases

## Tools

- `pretty_assertions`: Better assertion failure messages
- `proptest`: Property-based testing for edge cases
- `criterion`: Statistical benchmarking
