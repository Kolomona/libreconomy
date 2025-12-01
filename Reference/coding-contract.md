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
- All public APIs documented with `///`
- Module-level docs with `//!`
- Examples in docs for complex functions
- Update README.md for breaking changes

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
- [ ] Property tests added for numeric logic
- [ ] No unwrap/panic/expect in library code
- [ ] Uses Result<T, E> for fallible operations
- [ ] Follows ECS pattern (Components + Systems)
- [ ] Public APIs documented
- [ ] Uses type-safe wrappers (AgentId, not u64)
- [ ] Feature flags used for optional code
- [ ] Benchmarks added if performance-critical

## KEY PRINCIPLE

**If uncertain whether something violates this contract, STOP and ASK the human first.**

---

**Contract Acknowledgment**: By working on this project, you agree to follow these standards without exception unless explicitly authorized by the human to deviate.

**Last Updated**: Initial version
**Effective**: All development going forward