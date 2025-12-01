# Phase Plan: Agent Entity and Core Components Implementation

## Problem Description
Implement the foundational agent entity and its core components (Needs, Inventory, Wallet) in ECS. This enables basic simulation and testing, forming the basis for all future systems.

---

## Phase 1: Agent Entity Structure
- [x] Define Agent entity type in ECS
- [x] Ensure unique AgentId assignment
	- Implemented `AgentId` newtype and `Agent` component
	- Added `AgentIdAllocator` resource with safe, overflow-checked allocation
	- Added simple unit tests validating uniqueness and ECS registration

## Phase 2: Core Agent Components
- [x] Implement Needs component (thirst, hunger)
	- Added `Needs` with serde support and clamp helpers
	- Introduced `MIN_NEEDS`/`MAX_NEEDS` constants and simple decay-safe clamping
	- Unit tests validate clamping and bounds
- [x] Implement Inventory component (references to Item entities)
	- Implemented `Inventory` as item_id -> quantity map with safe add/remove
	- Accessors avoid panics and return 0 for missing items
	- Unit tests validate add/remove saturation and zero behavior
- [x] Implement Wallet component (currency balance)
	- Implemented `Wallet` with non-negative balance guarantees
	- Safe `deposit`/`withdraw` that ignore invalid inputs and prevent negatives
	- Unit tests validate creation, deposit/withdraw semantics

## Phase 3: Agent Creation Logic
- [x] Implement agent creation function
	- Added `create_agent()` for creating agents with default components
	- Added `create_agent_with_needs()` for custom needs
	- Added `create_agent_with_wallet()` for custom wallet
	- Added `create_agent_custom()` for fully customized agents
	- Default values: thirst=50.0, hunger=50.0, currency=100.0, empty inventory
- [x] Assign default components to new agents
	- All creation functions automatically assign Agent, Needs, Inventory, and Wallet components
	- AgentId is automatically allocated from AgentIdAllocator resource
- [x] Validate agent creation with unit tests
	- Unit tests in `src/agent/creation.rs` module
	- Integration tests in `tests/agent_creation_phase3.rs`
	- Example showcase tests in `tests/basic_simulation.rs`
	- Updated `examples/basic_simulation.rs` to demonstrate Phase 3 functionality

## Phase 4: Agent Lifecycle Management
- [ ] Implement agent removal function
- [ ] Ensure ECS world updates on agent removal
- [ ] Validate agent removal with unit tests

## Phase 5: ECS Integration
- [ ] Register agent and core components with ECS world
- [ ] Test ECS queries for agents and components

## Phase 6: Unit Testing
- [ ] Write unit tests for agent creation
- [ ] Write unit tests for component assignment
- [ ] Write unit tests for agent removal

---

Each phase is self-contained and can be executed in a single session by an AI agent. All implementation must conform to coding standards in Reference/coding-contract.md.
