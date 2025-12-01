//! Agent identity types and allocators
//! Data-only module for AgentId and its allocator resource

use serde::{Deserialize, Serialize};

/// Strongly-typed identifier for agents
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

/// Errors that can occur when allocating a new AgentId
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentIdError {
    /// Exhausted all available AgentId values
    Overflow,
}

impl core::fmt::Display for AgentIdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AgentIdError::Overflow => write!(f, "AgentId overflow: no more IDs available"),
        }
    }
}

impl std::error::Error for AgentIdError {}

/// Resource responsible for generating unique AgentId values
///
/// Insert this into the ECS world as a resource when you need to
/// allocate IDs during entity creation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentIdAllocator {
    next: u64,
}

impl AgentIdAllocator {
    /// Create a new allocator starting at 1
    pub fn new() -> Self {
        Self { next: 1 }
    }

    /// Allocate the next unique AgentId
    pub fn allocate(&mut self) -> Result<AgentId, AgentIdError> {
        let id = self.next;
        // checked_add to prevent overflow in release builds as well
        self.next = self.next.checked_add(1).ok_or(AgentIdError::Overflow)?;
        Ok(AgentId(id))
    }

    /// Peek the next value without consuming it (useful for testing)
    pub fn peek(&self) -> AgentId {
        AgentId(self.next)
    }
}

impl Default for AgentIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}
