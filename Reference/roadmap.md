# libreconomy Development Roadmap

# Agents System
- [x] Define agent entity structure
- [x] Implement core agent components (Needs, Inventory, Wallet, Skills, Knowledge, Employment, Preferences)
- [x] Implement agent creation logic
- [x] Implement agent lifecycle management (creation, removal)
- [x] Integrate agents with ECS world
- [x] Add unit tests for agent logic

- [ ] Market System
	- [ ] Define market component data structures
	- [ ] Implement buyer/seller matching logic
	- [ ] Implement trade transaction logic
	- [ ] Implement price discovery algorithm
	- [ ] Integrate market system with ECS world
	- [ ] Add unit tests for market logic

- [ ] Labor System
	- [ ] Define employment/gig work components
	- [ ] Implement job matching logic
	- [ ] Implement employer/employee relationship management
	- [ ] Integrate labor system with ECS world
	- [ ] Add unit tests for labor logic

- [ ] Production System
	- [ ] Define production/recipe components
	- [ ] Implement goods transformation logic
	- [ ] Implement skill/tool requirements
	- [ ] Integrate production system with ECS world
	- [ ] Add unit tests for production logic

- [ ] Decision System
	- [ ] Define trait for agent decision-making
	- [ ] Implement utility maximization strategy
	- [ ] Implement pluggable decision strategies
	- [ ] Integrate decision system with ECS world
	- [ ] Add unit tests for decision logic

- [ ] Learning System
	- [ ] Define knowledge component structure
	- [ ] Implement agent price/trade knowledge update logic
	- [ ] Integrate learning system with ECS world
	- [ ] Add unit tests for learning logic

- [ ] Negotiation System
	- [ ] Implement bartering logic
	- [ ] Implement price negotiation algorithm
	- [ ] Integrate negotiation system with ECS world
	- [ ] Add unit tests for negotiation logic

- [ ] Simulation Loop Integration
	- [ ] Connect all systems to ECS world
	- [ ] Implement simulation step/loop function
	- [ ] Expose simulation control via FFI
	- [ ] Add unit tests for simulation loop

- [ ] FFI API Expansion
	- [ ] Expose agent creation functions
	- [ ] Expose simulation control functions
	- [ ] Expose state query functions
	- [ ] Add FFI unit/integration tests

- [ ] Serialization/Deserialization
	- [ ] Implement save/load for simulation state
	- [ ] Add unit tests for serialization logic

- [ ] Graph Structures
	- [ ] Integrate petgraph for trade networks
	- [ ] Integrate petgraph for production chains
	- [ ] Integrate petgraph for employment relationships
	- [ ] Add unit tests for graph logic

- [ ] Testing
	- [ ] Write unit tests for all systems
	- [ ] Add property-based tests for complex logic
	- [ ] Add integration tests for FFI and simulation

- [ ] Examples
	- [ ] Add example for agent creation
	- [ ] Add example for market transactions
	- [ ] Add example for labor and production
	- [ ] Add example for simulation loop usage

- [ ] Documentation
	- [ ] Document all public APIs
	- [ ] Add module-level documentation
	- [ ] Add API usage examples to docs

- [ ] Performance Optimization
	- [ ] Profile for low-power devices
	- [ ] Minimize allocations in hot loops
	- [ ] Reuse collections where possible

- [ ] FFI Bindings Validation
	- [ ] Validate cbindgen C header generation
	- [ ] Validate uniffi bindings for all languages
	- [ ] Add FFI usage tests for each language
