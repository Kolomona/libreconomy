pub mod components;

pub mod identity;

pub mod creation;

pub use identity::{AgentId, AgentIdAllocator, AgentIdError};
pub use creation::{create_agent, create_agent_with_needs, create_agent_with_wallet, create_agent_custom, remove_agent};

// TODO: Add agent systems and logic
