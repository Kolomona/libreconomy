# Economy Simulator Library - Project Summary

## Project Goal

Create a cross-platform, open-source economy simulator library that can be integrated into various applications (video games, simulations, etc.) and run efficiently on low-power devices like Raspberry Pi Zero.

## Core Requirements

- **Open source**
- **Cross-platform** compatibility
- **High performance** (native code execution)
- **Low-power efficient** (suitable for Raspberry Pi Zero)
- **Easy integration** into multiple application types and languages

## Technology Stack

### Primary Language

**Rust** - chosen for:

- Native performance (matches C/C++)
- Memory safety without runtime overhead
- No garbage collector (critical for low-power devices)
- Excellent cross-platform tooling
- Strong FFI capabilities

### FFI/Binding Strategy

**Dual approach for maximum compatibility:**

1. **cbindgen** - Generate C/C++ headers
    
    - Provides standard C API
    - Works with game engines (Unity, Unreal, Godot)
    - Compatible with any language supporting C FFI
    - Maximum performance, minimal overhead
2. **uniffi** - Generate high-level language bindings
    
    - Creates idiomatic bindings for Python, Swift, Kotlin, Ruby
    - Automatically handles complex types
    - Better developer experience for scripting languages
    - Useful for mobile apps and rapid prototyping

### Dependencies

**Required:**

- **serde** - Serialization for save/load functionality

**Core features (always included):**

- **specs** - Entity Component System for managing economic entities
- **petgraph** - Graph structures for trade networks and dependencies

**Optional (disabled by default):**

- **rayon** - Parallelism for larger simulations on multi-core systems
    - Disabled by default since Raspberry Pi Zero is single-core
    - Can be enabled via feature flag for desktop/server deployments

### Cargo Features Structure

```toml
[features]
default = []  # Minimal build for embedded
parallel = ["rayon"]  # Enable for multi-core systems
```

## Economic Model Design

### Agent-Based System

Agents (players and NPCs) drive the entire economy through their individual decisions and interactions.

**Core Principle: Subjective Value Theory**

- Value is determined by each agent's internal needs and motivations
- Example: A thirsty agent values water more highly than a hydrated agent
- Agents willing to pay more or work harder for goods that satisfy urgent needs

### Key Economic Features

**1. Trading Mechanisms**

- **Bartering** - Direct exchange of goods/services
- **Currency-based** - Monetary transactions
- Both systems coexist and agents choose based on circumstances

**2. Labor Markets**

- **Regular employment** - Ongoing employer/employee relationships
- **Gig work** - Short-term contract jobs
- **One-off jobs** - Single task transactions
- Agents can both hire and be hired

**3. Production System**

- Agents create finished goods from raw materials
- Production mechanics to be designed (recipes, skills, tools, time?)
- Creates supply chains and specialization opportunities

**4. Price Discovery**

- **No omniscient pricing** - Agents don't automatically know all prices
- **Discovery through interaction** - Agents learn prices by trading, asking, observing
- Creates information asymmetry and realistic market dynamics
- Enables emergent phenomena (arbitrage, rumors, local markets)

**5. Decision-Making Architecture**

- **Initial implementation**: Simple utility maximization
    - Agents calculate utility based on needs vs. costs
    - Choose actions that maximize expected satisfaction
- **Pluggable design**: Decision-making system easily swappable
    - Interface/trait-based architecture
    - Allows future evolution to more complex AI
    - Could later support: learning algorithms, personality types, bounded rationality, heuristics

### Architecture Approach

**Entity-Component-System (specs) structure:**

```
Agent Entity:
├─ Needs Component (thirst, hunger, shelter, etc.)
├─ Inventory Component (goods owned)
├─ Wallet Component (currency)
├─ Knowledge Component (known prices, trade partners)
├─ Skills Component (production capabilities)
├─ Employment Component (job status, employer/employees)
└─ Preferences Component (utility functions, risk tolerance)

Systems:
├─ Need Decay System (needs increase over time)
├─ Decision System (pluggable - evaluates options)
├─ Market System (matches buyers/sellers for goods)
├─ Labor System (matches employers/workers)
├─ Production System (transforms materials into goods)
├─ Learning System (agents update price knowledge)
└─ Negotiation System (handles bartering)
```

**Graph structures (petgraph) for:**

- Trade networks and relationships
- Production chains (raw materials → intermediate → finished goods)
- Employment relationships
- Information flow networks

## Overall Architecture

Core simulation logic (Rust + specs + petgraph) → C API layer (via cbindgen) → High-level bindings (via uniffi) → Consumer applications

This provides a performant, emergent economy driven by agent behavior that's both realistic and computationally efficient for embedded/game use cases, with convenient bindings for scripting/mobile applications.