# libreconomy Development Contract

**VERSION 1.0 - This contract defines non-negotiable standards for this project**

## CRITICAL RULES - NEVER VIOLATE WITHOUT HUMAN APPROVAL

### 1. TEST-DRIVEN DEVELOPMENT (MANDATORY)
- ❌ **NO CODE WITHOUT TESTS FIRST**
- Write failing test → Implement → Refactor → Add property tests
- All tests must pass before any commit
- Property tests required for all numeric/economic logic
- Benchmarks required for performance-critical code

### 2. ARCHITECTURE (FIXED)
- **Language**: Rust 2021
- **Pattern**: Entity-Component-System (specs)
- **Data in Components, Logic in Systems** - NO EXCEPTIONS
- **FFI**: cbindgen (C/C++) + uniffi (high-level languages)
- **Dependencies**: serde, specs, petgraph | Optional: rayon (feature-gated)

### 3. PERFORMANCE (RASPBERRY PI ZERO CONSTRAINT)
- Must run efficiently on single-core ARM
- No garbage collection, minimal allocations
- No unwrap/panic in library code
- Benchmark before optimizing
- Default features must work on Pi Zero

### 4. CODE STANDARDS (ENFORCE)
```rust
✅ Type safety: struct AgentId(u64), not u64
✅ Error handling: Result<T, E>, never unwrap()
✅ Traits for abstraction: trait DecisionMaker {...}
✅ ECS separation: Components (data) + Systems (logic)
✅ Feature flags: #[cfg(feature = "parallel")]

❌ No panic/unwrap/expect in library code
❌ No unsafe without justification + docs
❌ No global state or singletons
❌ No hardcoded behavior - keep data-driven
❌ No tight coupling between systems
```

### 5. ECONOMIC MODEL (IMMUTABLE)
- Agent-based with subjective value theory
- Trading: Both barter AND currency
- Labor: Employment, gigs, one-off jobs
- Production: Agents create goods from materials
- Information: Discovered through interaction (not omniscient)
- Decisions: Pluggable via traits (start with utility maximization)

### 6. TESTING REQUIREMENTS
```rust
// REQUIRED test structure
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;  // ALWAYS use this
    
    #[test]
    fn test_name() { /* unit test */ }
}

// REQUIRED for numeric logic
proptest! {
    #[test]
    fn property_name(val in range) { /* property test */ }
}
```

### 7. DOCUMENTATION (MANDATORY)

Documentation follows a multi-layered approach. All layers are required.

#### 7.1 Inline Rust Documentation (PRIMARY - cargo doc)
```rust
//! Module-level documentation
//! 
//! Explains module purpose, core concepts, usage patterns.
//! Include quick-start example demonstrating typical usage.

/// Function documentation
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Example
/// ```
/// use libreconomy::*;
/// let result = function_name(param);
/// ```
///
/// # Panics
/// Document panic conditions
///
/// # Errors  
/// Document error conditions for Result types
pub fn function_name(param: Type) -> ReturnType { }
```

**Rules:**
- All public items (`pub`) MUST have `///` documentation
- All modules MUST have `//!` documentation at top
- Complex functions MUST include working examples in docs
- Doc examples MUST be tested (`cargo test --doc` must pass)
- Use `# Example` section liberally for non-trivial APIs
- Document panic conditions with `# Panics`
- Document error cases with `# Errors` for Result types

#### 7.2 User Guide (docs/GUIDE.md)
**Purpose:** Narrative tutorials, not API reference duplication

**Required sections:**
- Quick Start (copy-paste working example)
- Core Concepts (ECS, agents, components)
- Common Patterns (how to do X, Y, Z)
- Working examples that demonstrate workflow

**Rules:**
- Do NOT duplicate API docs - link to `cargo doc` instead
- Focus on teaching concepts and patterns
- All code examples must be complete and runnable
- Update when adding major features or changing workflows

#### 7.3 FFI Documentation (docs/api/FFI.md)
**Purpose:** Non-Rust language integration

**Required sections per language:**
- Build instructions
- Complete working example
- Platform-specific notes (iOS, Android, etc.)
- Linking/deployment instructions
- Troubleshooting common issues

**Rules:**
- Update when FFI surface changes
- Include examples in target language (Python, Swift, Kotlin)
- Document platform-specific quirks
- Keep synchronized with uniffi exports

#### 7.4 README.md
**Purpose:** Project overview and navigation hub

**Required content:**
- Current implementation status (what works NOW)
- Quick example showcasing core functionality
- Links to all documentation layers
- Clear roadmap (what's planned)

**Rules:**
- Update when completing major features/phases
- Keep "Current Implementation" section accurate
- Do NOT document future features as if they exist
- Link to other docs, don't duplicate them

#### 7.5 Documentation Testing
```bash
# All must pass before commit:
cargo test --doc          # Test inline examples
cargo doc --no-deps       # Verify docs build without errors
```

#### 7.6 Documentation Updates Required When:
- Adding public API → Inline docs + User Guide update
- Changing FFI surface → FFI.md update
- Completing major feature → README.md "Current Implementation" update
- Breaking changes → All affected docs + README.md
- New usage patterns → User Guide update

#### 7.7 Documentation Anti-Patterns (FORBIDDEN)
❌ Duplicating API details in multiple places (causes drift)
❌ Manual API docs in docs/api/ folder (use cargo doc)
❌ Documenting planned features as implemented
❌ Untested code examples in documentation
❌ Vague descriptions without examples for complex APIs
❌ Missing panic/error documentation

### 8. NAMING & STYLE (STRICT)
```rust
struct TypeName { }           // CamelCase
fn function_name() { }        // snake_case
const MAX_VALUE: f32 = 100.0; // SCREAMING_SNAKE_CASE
mod module_name;              // snake_case
```

## CONSULTATION REQUIRED FOR

- Adding new dependencies (impacts binary size/compile time)
- Breaking backward compatibility (requires major version bump)
- Using `unsafe` code (requires thorough justification)
- Deviating from ECS pattern (Components + Systems)
- Changing core economic model principles
- Skipping TDD process for any reason
- Optimizing before benchmarking proves necessity

## VERIFICATION CHECKLIST

Before any code submission, verify:
- [ ] Tests written BEFORE implementation
- [ ] All tests pass (`cargo test`)
- [ ] Doc tests pass (`cargo test --doc`)
- [ ] Documentation builds (`cargo doc --no-deps`)
- [ ] Property tests added for numeric logic
- [ ] No unwrap/panic/expect in library code
- [ ] Uses Result<T, E> for fallible operations
- [ ] Follows ECS pattern (Components + Systems)
- [ ] All public APIs have `///` documentation with examples
- [ ] Module has `//!` documentation
- [ ] docs/GUIDE.md updated if workflow/patterns changed
- [ ] docs/api/FFI.md updated if FFI surface changed
- [ ] README.md updated if major feature completed
- [ ] Uses type-safe wrappers (AgentId, not u64)
- [ ] Feature flags used for optional code
- [ ] Benchmarks added if performance-critical

## FFI BINDINGS POLICY (MANDATORY)

The project uses metadata-based uniffi binding generation. Follow these rules precisely to ensure bindings are always generated and releases are reproducible.

1) Binding strategy
- C header: Generated with cbindgen during release.
- High-level languages (Python/Swift/Kotlin): Generated from compiled cdylib metadata via uniffi-bindgen (no UDL parsing in the critical path).

1.a) C header and `cbindgen.toml` (MANDATORY)
- The file `cbindgen.toml` MUST exist and be kept under version control.
- Header generation must be deterministic and stable across tool versions.
- Required minimal settings in `cbindgen.toml`:
    - `include_guard = "LIBRECONOMY_H"`
    - `pragma_once = true`
    - `sys_includes = ["stdint.h", "stdbool.h", "stdlib.h"]`
    - `usize_is_size_t = true`
    - `cpp_compat = true`
    - `documentation = false`
    - `[parse] parse_deps = false`
- Do NOT rely on per-item renaming in `cbindgen` `config`; keep Rust symbol names stable and intentional.
- Breaking changes to exported C symbols require a major version bump.

2) Crate setup
- Cargo.toml must include:
    - `uniffi = { version = "0.28", features = ["build"] }`
    - Library types: `rlib` and `cdylib` (for tests and FFI).

3) Exporting APIs
- Annotate every exported function or method with `#[uniffi::export]`.
- Place `uniffi::setup_scaffolding!()` exactly once in the crate root (`src/lib.rs`).
- Keep FFI surface minimal and stable. Use only uniffi-supported types:
    - Primitives: `bool`, `i{8,16,32,64}`, `u{8,16,32,64}`, `f32`, `f64`
    - `String`, `Vec<T>`, `Option<T>`, `Result<T, E>`
    - Custom types: derive `uniffi::Record`, `uniffi::Enum`, `uniffi::Error` as needed
- Do not panic across FFI. Use `Result<T, E>` with a `#[derive(uniffi::Error)]` error type.
- Keep FFI thin; all domain logic remains in pure Rust.

4) Release requirements (STRICT)
- The release script MUST:
    - Build in `--release` mode
    - Generate `libreconomy.h` via `cbindgen`
    - Generate Python/Swift/Kotlin bindings via `uniffi-bindgen generate --library` from the compiled `cdylib`
    - FAIL if any binding generation step fails

5) Version alignment
- The Rust `uniffi` crate version and the `uniffi-bindgen` CLI must be aligned (currently 0.28.x).
- Upgrades to newer uniffi versions must be explicit and include TDD updates to the FFI layer.

6) Examples
```toml
# Cargo.toml
[dependencies]
uniffi = { version = "0.28", features = ["build"] }
```

```rust
// src/lib.rs (crate root)
uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn libreconomy_version() -> String { "0.0.1".to_string() }

#[uniffi::export]
pub fn get_agent_count() -> u32 { 0 }
```

7) UDL files
- Optional for documentation only; NOT required for binding generation.
- Do not block releases on UDL parsing. The canonical source for bindings is the compiled cdylib metadata.

## TESTING ADDENDUM FOR FFI

- Add a minimal FFI smoke test alongside unit tests:
    - Run the release script (or a binding generation step) in CI.
    - For Python, import the generated module and call a trivial function (e.g., `libreconomy_version()`).
- Keep FFI tests simple and fast; do not duplicate domain tests across languages.

## KEY PRINCIPLE

**If uncertain whether something violates this contract, STOP and ASK the human first.**

---

**Contract Acknowledgment**: By working on this project, you agree to follow these standards without exception unless explicitly authorized by the human to deviate.

**Last Updated**: Initial version
**Effective**: All development going forward