// World Query trait for libreconomy
//
// This trait defines the interface between libreconomy (economic simulation)
// and applications (spatial/physical simulation). Applications implement this
// trait to provide libreconomy with spatial information without the library
// needing to know about the specific world structure.
//
// # Design Philosophy
//
// libreconomy is a pure economic simulation library - it handles economic logic
// (decision-making, trading, production), NOT spatial/physical simulation.
// The WorldQuery trait is the boundary that separates these concerns:
//
// - Applications own: Entity positions, terrain, pathfinding, rendering
// - libreconomy owns: Utility calculation, preferences, trading logic
//
// # Example
//
// ```rust
// use libreconomy::world_query::{WorldQuery, ResourceLocation};
// use libreconomy::agent::AgentId;
//
// struct MyGameWorld {
//     entities: Vec<(AgentId, f32, f32)>, // id, x, y
//     water_sources: Vec<(f32, f32)>,
// }
//
// impl WorldQuery for MyGameWorld {
//     fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId> {
//         // Find agents near the given agent
//         // Return up to max_count closest agents
//         vec![]
//     }
//
//     fn get_nearby_resources(
//         &self,
//         agent: AgentId,
//         resource_type: &str,
//         max_radius: f32
//     ) -> Vec<ResourceLocation> {
//         // Find resources of given type within radius
//         vec![]
//     }
//
//     fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool {
//         // Check if agents are close enough to interact
//         false
//     }
// }
// ```

use crate::agent::AgentId;
use serde::{Deserialize, Serialize};

/// Location of a resource in the world (x, y coordinates and distance)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLocation {
    /// X coordinate in world space
    pub x: f32,
    /// Y coordinate in world space
    pub y: f32,
    /// Distance from the querying agent (for convenience)
    pub distance: f32,
}

impl ResourceLocation {
    /// Create a new resource location
    pub fn new(x: f32, y: f32, distance: f32) -> Self {
        Self { x, y, distance }
    }
}

/// Trait for querying spatial information from the application's world
///
/// Applications implement this trait to provide libreconomy with the spatial
/// context it needs for decision-making. The library calls these methods to:
/// - Find nearby agents for social interactions, trading, hunting
/// - Find nearby resources (water, food, materials) for survival needs
/// - Check if two agents can interact (proximity, line of sight, etc.)
///
/// # Implementation Notes
///
/// - Results should be sorted by distance (closest first) when relevant
/// - Return empty vectors if no matches found (don't return None)
/// - Performance matters: These methods may be called frequently
/// - Consider using spatial hashing or other optimizations
///
/// # Resource Types
///
/// Resource type strings are application-defined. Common examples:
/// - "water" - Water sources for drinking
/// - "grass" - Vegetation for herbivores
/// - "tree" - Trees for lumbering
/// - "ore" - Mining resources
///
/// Applications can define custom resource types as needed.
pub trait WorldQuery {
    /// Get agents near the given agent
    ///
    /// Returns a list of agent IDs sorted by distance (closest first).
    /// The list will contain at most `max_count` agents.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to search around
    /// * `max_count` - Maximum number of agents to return
    ///
    /// # Returns
    ///
    /// Vector of agent IDs, sorted by distance (closest first).
    /// Empty vector if no agents found nearby.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let nearby = world_query.get_nearby_agents(my_agent, 10);
    /// // Returns up to 10 closest agents
    /// ```
    fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId>;

    /// Get resources of a specific type near the given agent
    ///
    /// Returns a list of resource locations sorted by distance (closest first).
    /// Only resources within `max_radius` world units are returned.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to search around
    /// * `resource_type` - Type of resource to find (e.g., "water", "grass")
    /// * `max_radius` - Maximum search radius in world units
    ///
    /// # Returns
    ///
    /// Vector of ResourceLocation, sorted by distance (closest first).
    /// Empty vector if no matching resources found within radius.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let water = world_query.get_nearby_resources(my_agent, "water", 500.0);
    /// if let Some(closest) = water.first() {
    ///     println!("Closest water at ({}, {}) distance {}",
    ///              closest.x, closest.y, closest.distance);
    /// }
    /// ```
    fn get_nearby_resources(
        &self,
        agent: AgentId,
        resource_type: &str,
        max_radius: f32,
    ) -> Vec<ResourceLocation>;

    /// Check if two agents can interact
    ///
    /// Returns true if the agents are close enough to interact (trade,
    /// communicate, fight, etc.). The specific distance threshold is
    /// application-defined.
    ///
    /// # Arguments
    ///
    /// * `agent1` - First agent
    /// * `agent2` - Second agent
    ///
    /// # Returns
    ///
    /// true if agents can interact, false otherwise
    ///
    /// # Example
    ///
    /// ```ignore
    /// if world_query.can_interact(buyer, seller) {
    ///     // Execute trade
    /// }
    /// ```
    fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock WorldQuery implementation for testing
    struct MockWorldQuery {
        agents: Vec<(AgentId, f32, f32)>, // id, x, y
        resources: Vec<(String, f32, f32)>, // type, x, y
        interaction_distance: f32,
    }

    impl MockWorldQuery {
        fn new() -> Self {
            Self {
                agents: Vec::new(),
                resources: Vec::new(),
                interaction_distance: 10.0,
            }
        }

        fn add_agent(&mut self, id: AgentId, x: f32, y: f32) {
            self.agents.push((id, x, y));
        }

        fn add_resource(&mut self, resource_type: &str, x: f32, y: f32) {
            self.resources.push((resource_type.to_string(), x, y));
        }

        fn distance(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
            ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
        }

        fn get_agent_position(&self, agent: AgentId) -> Option<(f32, f32)> {
            self.agents
                .iter()
                .find(|(id, _, _)| *id == agent)
                .map(|(_, x, y)| (*x, *y))
        }
    }

    impl WorldQuery for MockWorldQuery {
        fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId> {
            let (ax, ay) = match self.get_agent_position(agent) {
                Some(pos) => pos,
                None => return Vec::new(),
            };

            let mut nearby: Vec<(AgentId, f32)> = self
                .agents
                .iter()
                .filter(|(id, _, _)| *id != agent) // Exclude self
                .map(|(id, x, y)| {
                    let dist = self.distance(ax, ay, *x, *y);
                    (*id, dist)
                })
                .collect();

            // Sort by distance
            nearby.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            // Take up to max_count
            nearby.iter().take(max_count).map(|(id, _)| *id).collect()
        }

        fn get_nearby_resources(
            &self,
            agent: AgentId,
            resource_type: &str,
            max_radius: f32,
        ) -> Vec<ResourceLocation> {
            let (ax, ay) = match self.get_agent_position(agent) {
                Some(pos) => pos,
                None => return Vec::new(),
            };

            let mut nearby: Vec<ResourceLocation> = self
                .resources
                .iter()
                .filter(|(rtype, _, _)| rtype == resource_type)
                .map(|(_, x, y)| {
                    let dist = self.distance(ax, ay, *x, *y);
                    ResourceLocation::new(*x, *y, dist)
                })
                .filter(|loc| loc.distance <= max_radius)
                .collect();

            // Sort by distance
            nearby.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

            nearby
        }

        fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool {
            let (x1, y1) = match self.get_agent_position(agent1) {
                Some(pos) => pos,
                None => return false,
            };

            let (x2, y2) = match self.get_agent_position(agent2) {
                Some(pos) => pos,
                None => return false,
            };

            self.distance(x1, y1, x2, y2) <= self.interaction_distance
        }
    }

    #[test]
    fn test_resource_location_creation() {
        let loc = ResourceLocation::new(10.0, 20.0, 5.0);
        assert_eq!(loc.x, 10.0);
        assert_eq!(loc.y, 20.0);
        assert_eq!(loc.distance, 5.0);
    }

    #[test]
    fn test_get_nearby_agents() {
        let mut world = MockWorldQuery::new();
        let agent0 = AgentId(0);
        let agent1 = AgentId(1);
        let agent2 = AgentId(2);
        let agent3 = AgentId(3);

        world.add_agent(agent0, 0.0, 0.0);
        world.add_agent(agent1, 10.0, 0.0);
        world.add_agent(agent2, 5.0, 0.0);
        world.add_agent(agent3, 50.0, 0.0);

        let nearby = world.get_nearby_agents(agent0, 10);
        // Should return all others, sorted by distance
        assert_eq!(nearby.len(), 3);
        assert_eq!(nearby[0], agent2); // Distance 5
        assert_eq!(nearby[1], agent1); // Distance 10
        assert_eq!(nearby[2], agent3); // Distance 50

        // Test max_count limit
        let nearby_limited = world.get_nearby_agents(agent0, 2);
        assert_eq!(nearby_limited.len(), 2);
        assert_eq!(nearby_limited[0], agent2);
        assert_eq!(nearby_limited[1], agent1);
    }

    #[test]
    fn test_get_nearby_resources() {
        let mut world = MockWorldQuery::new();
        let agent = AgentId(0);

        world.add_agent(agent, 0.0, 0.0);
        world.add_resource("water", 10.0, 0.0);
        world.add_resource("water", 5.0, 0.0);
        world.add_resource("grass", 3.0, 0.0);
        world.add_resource("water", 100.0, 0.0);

        // Find water within 50 units
        let water = world.get_nearby_resources(agent, "water", 50.0);
        assert_eq!(water.len(), 2); // Two water sources within 50
        assert_eq!(water[0].distance, 5.0); // Closest first
        assert_eq!(water[1].distance, 10.0);

        // Find grass
        let grass = world.get_nearby_resources(agent, "grass", 50.0);
        assert_eq!(grass.len(), 1);
        assert_eq!(grass[0].distance, 3.0);

        // No results beyond radius
        let far_water = world.get_nearby_resources(agent, "water", 8.0);
        assert_eq!(far_water.len(), 1); // Only the 5.0 distance water
    }

    #[test]
    fn test_can_interact() {
        let mut world = MockWorldQuery::new();
        world.interaction_distance = 10.0;

        let agent1 = AgentId(1);
        let agent2 = AgentId(2);
        let agent3 = AgentId(3);

        world.add_agent(agent1, 0.0, 0.0);
        world.add_agent(agent2, 5.0, 0.0); // Within 10
        world.add_agent(agent3, 20.0, 0.0); // Beyond 10

        assert!(world.can_interact(agent1, agent2));
        assert!(!world.can_interact(agent1, agent3));
    }

    #[test]
    fn test_get_nearby_agents_excludes_self() {
        let mut world = MockWorldQuery::new();
        let agent = AgentId(0);
        world.add_agent(agent, 0.0, 0.0);

        let nearby = world.get_nearby_agents(agent, 10);
        assert_eq!(nearby.len(), 0); // Should not include self
    }

    #[test]
    fn test_get_nearby_resources_empty() {
        let mut world = MockWorldQuery::new();
        let agent = AgentId(0);
        world.add_agent(agent, 0.0, 0.0);

        let water = world.get_nearby_resources(agent, "water", 100.0);
        assert_eq!(water.len(), 0); // No resources added
    }

    #[test]
    fn test_can_interact_missing_agent() {
        let world = MockWorldQuery::new();
        let agent1 = AgentId(1);
        let agent2 = AgentId(2);

        assert!(!world.can_interact(agent1, agent2)); // No agents added
    }
}
