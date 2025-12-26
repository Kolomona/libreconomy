// Utility-based decision making system
//
// This module implements a utility maximizer that evaluates multiple possible
// actions and selects the one with the highest utility score. It's based on
// the JavaScript stub from libreterra but implemented in pure Rust.

use crate::{Agent, AgentId, Needs, SpeciesComponent, DietType};
use crate::decision::{DecisionOutput, Intent};
use crate::world_query::WorldQuery;
use specs::prelude::*;

/// Configuration thresholds for decision-making
///
/// These thresholds determine when an agent considers a need "high" enough
/// to warrant seeking satisfaction. Lower thresholds make agents more
/// proactive, higher thresholds make them wait until needs are more urgent.
#[derive(Debug, Clone)]
pub struct DecisionThresholds {
    /// Thirst level at which agent seeks water urgently (0-100)
    pub critical_thirst: f32,
    /// Thirst level at which agent starts seeking water (0-100)
    pub high_thirst: f32,
    /// Hunger level at which agent seeks food urgently (0-100)
    pub critical_hunger: f32,
    /// Hunger level at which agent starts seeking food (0-100)
    pub high_hunger: f32,
    /// Tiredness level at which agent must rest (0-100)
    pub critical_tiredness: f32,
    /// Tiredness level at which agent should rest (0-100)
    pub high_tiredness: f32,
}

impl Default for DecisionThresholds {
    fn default() -> Self {
        Self {
            critical_thirst: 80.0,
            high_thirst: 60.0,
            critical_hunger: 70.0,
            high_hunger: 50.0,
            critical_tiredness: 85.0,
            high_tiredness: 70.0,
        }
    }
}

/// Weights for utility calculation
///
/// These weights determine how much each factor contributes to the final
/// utility score. Higher weights make that factor more important in
/// decision-making.
#[derive(Debug, Clone)]
pub struct UtilityWeights {
    /// Multiplier for survival-critical needs (hunger, thirst)
    pub survival: f32,
    /// Multiplier for comfort needs (tiredness)
    pub comfort: f32,
    /// Multiplier for distance efficiency (prefer closer resources)
    pub efficiency: f32,
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            survival: 2.0,
            comfort: 1.0,
            efficiency: 0.5,
        }
    }
}

/// Utility-based decision maker
///
/// Evaluates all possible actions and selects the one with the highest
/// utility score. This is the primary decision-making algorithm in libreconomy.
///
/// # Algorithm
///
/// 1. Read agent's current needs (hunger, thirst, tiredness)
/// 2. Query world for nearby resources and agents
/// 3. Calculate utility for each possible intent:
///    - SEEK_WATER: if thirsty, find nearest water
///    - SEEK_FOOD: if hungry, find nearest food (species-dependent)
///    - REST: if tired, sleep in place
///    - WANDER: default low-utility action
/// 4. Return intent with highest utility
///
/// # Utility Calculation
///
/// ```text
/// urgency = need_value / 100.0
/// distance_factor = max(0, 1 - distance / max_radius)
/// utility = urgency * survival_weight + distance_factor * efficiency_weight
/// ```
///
/// # Example
///
/// ```ignore
/// use libreconomy::decision::UtilityMaximizer;
/// use specs::World;
///
/// let mut world = World::new();
/// // ... setup world, register components ...
///
/// let decision_maker = UtilityMaximizer::default();
/// let decision = decision_maker.decide(agent_entity, &world, &my_world_query);
///
/// match decision {
///     DecisionOutput::Intent(Intent::SeekItem { item_type, urgency }) => {
///         println!("Agent seeking {} with urgency {}", item_type, urgency);
///     }
///     _ => {}
/// }
/// ```
pub struct UtilityMaximizer {
    /// Thresholds for triggering decisions
    pub thresholds: DecisionThresholds,
    /// Weights for utility calculation
    pub weights: UtilityWeights,
    /// Maximum radius to search for resources (world units)
    pub resource_search_radius: f32,
}

impl Default for UtilityMaximizer {
    fn default() -> Self {
        Self {
            thresholds: DecisionThresholds::default(),
            weights: UtilityWeights::default(),
            resource_search_radius: 1000.0,
        }
    }
}

impl UtilityMaximizer {
    /// Create a new UtilityMaximizer with custom configuration
    pub fn new(
        thresholds: DecisionThresholds,
        weights: UtilityWeights,
        resource_search_radius: f32,
    ) -> Self {
        Self {
            thresholds,
            weights,
            resource_search_radius,
        }
    }

    /// Make a decision for the given agent
    ///
    /// # Arguments
    ///
    /// * `agent` - The entity to make a decision for
    /// * `world` - The ECS world containing all components
    /// * `world_query` - Application-provided spatial query interface
    ///
    /// # Returns
    ///
    /// A DecisionOutput containing the chosen Intent
    ///
    /// # Panics
    ///
    /// Panics if the agent entity doesn't have required components (Needs).
    pub fn decide(
        &self,
        agent: Entity,
        world: &World,
        world_query: &dyn WorldQuery,
    ) -> DecisionOutput {
        // Read agent's needs
        let needs_storage = world.read_storage::<Needs>();
        let needs = needs_storage
            .get(agent)
            .expect("Agent must have Needs component");

        // Get agent ID for spatial queries
        let agent_storage = world.read_storage::<Agent>();
        let agent_component = agent_storage
            .get(agent)
            .expect("Agent must have Agent component");
        let agent_id = agent_component.id;

        // Evaluate all possible intents
        let mut utilities: Vec<(Intent, f32, String)> = Vec::new();

        // Evaluate SEEK_WATER
        if needs.thirst > self.thresholds.high_thirst {
            if let Some((utility, reason)) = self.evaluate_seek_water(
                agent_id,
                needs.thirst,
                world_query,
            ) {
                utilities.push((
                    Intent::SeekItem {
                        item_type: "water".to_string(),
                        urgency: needs.thirst / 100.0,
                    },
                    utility,
                    reason,
                ));
            }
        }

        // Evaluate SEEK_FOOD (species-aware)
        if needs.hunger > self.thresholds.high_hunger {
            // Get species component (optional - defaults to omnivore if not present)
            let species_storage = world.read_storage::<SpeciesComponent>();
            let species = species_storage.get(agent);

            if let Some((utility, reason, item_type)) = self.evaluate_seek_food(
                agent_id,
                needs.hunger,
                species,
                world_query,
            ) {
                utilities.push((
                    Intent::SeekItem {
                        item_type,
                        urgency: needs.hunger / 100.0,
                    },
                    utility,
                    reason,
                ));
            }
        }

        // Evaluate REST
        if needs.tiredness > self.thresholds.high_tiredness {
            let urgency = needs.tiredness / 100.0;
            let utility = urgency * self.weights.comfort;
            utilities.push((
                Intent::Rest,
                utility,
                format!("Tiredness: {:.0}", needs.tiredness),
            ));
        }

        // Always include WANDER as fallback
        utilities.push((Intent::Wander, 0.1, "Exploring".to_string()));

        // Sort by utility (highest first)
        utilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Log decision (1% of the time to avoid spam)
        if rand::random::<f32>() < 0.01 {
            let (ref intent, utility, ref reason) = utilities[0];
            println!(
                "Agent {:?} decided: {} (utility: {:.2}, reason: {})",
                agent_id,
                intent.intent_type(),
                utility,
                reason
            );
        }

        // Return highest utility intent
        DecisionOutput::Intent(utilities[0].0.clone())
    }

    /// Evaluate utility of seeking water
    fn evaluate_seek_water(
        &self,
        agent_id: AgentId,
        thirst: f32,
        world_query: &dyn WorldQuery,
    ) -> Option<(f32, String)> {
        let urgency = thirst / 100.0;

        // Query for nearby water sources
        let water_sources = world_query.get_nearby_resources(
            agent_id,
            "water",
            self.resource_search_radius,
        );

        if let Some(closest) = water_sources.first() {
            // Calculate distance factor (closer is better)
            let distance_factor =
                (1.0 - (closest.distance / self.resource_search_radius)).max(0.0);

            // Combine urgency and efficiency
            let utility = urgency * self.weights.survival
                + distance_factor * self.weights.efficiency;

            let reason = format!(
                "Thirst: {:.0} (water at distance {:.0})",
                thirst, closest.distance
            );

            Some((utility, reason))
        } else {
            // No water found, but still urgent - create intent to wander toward water
            let utility = urgency * self.weights.survival;
            let reason = format!("Thirst: {:.0} (searching for water)", thirst);
            Some((utility, reason))
        }
    }

    /// Evaluate utility of seeking food
    fn evaluate_seek_food(
        &self,
        agent_id: AgentId,
        hunger: f32,
        species: Option<&SpeciesComponent>,
        world_query: &dyn WorldQuery,
    ) -> Option<(f32, String, String)> {
        let urgency = hunger / 100.0;

        // Determine what food sources this species can eat
        let food_items: Vec<&str> = match species {
            Some(s) => match &s.diet {
                DietType::Herbivore { preferred_plants } => {
                    if preferred_plants.is_empty() {
                        // Accept any plant
                        vec!["grass", "food"]
                    } else {
                        preferred_plants.iter().map(|s| s.as_str()).collect()
                    }
                }
                DietType::Carnivore { .. } => {
                    // Carnivores need to hunt (not implemented fully yet)
                    // For now, they can seek "rabbit_meat"
                    vec!["rabbit_meat"]
                }
                DietType::Omnivore { plants, .. } => {
                    if plants.is_empty() {
                        // Accept any plant
                        vec!["grass", "food"]
                    } else {
                        plants.iter().map(|s| s.as_str()).collect()
                    }
                }
            },
            None => {
                // No species component - default to omnivore behavior
                vec!["grass", "food"]
            }
        };

        // Try each food type and find the best option
        let mut best_option: Option<(f32, String, String)> = None;

        for food_type in &food_items {
            let food_sources = world_query.get_nearby_resources(
                agent_id,
                food_type,
                self.resource_search_radius,
            );

            if let Some(closest) = food_sources.first() {
                let distance_factor =
                    (1.0 - (closest.distance / self.resource_search_radius)).max(0.0);

                let utility = urgency * self.weights.survival
                    + distance_factor * self.weights.efficiency;

                let reason = format!(
                    "Hunger: {:.0} ({} at distance {:.0})",
                    hunger, food_type, closest.distance
                );

                // Keep the best option
                if best_option.is_none() || utility > best_option.as_ref().unwrap().0 {
                    best_option = Some((utility, reason, food_type.to_string()));
                }
            }
        }

        if let Some(option) = best_option {
            Some(option)
        } else {
            // No food found, but still urgent - wander to search
            let utility = urgency * self.weights.survival;
            let food_type = food_items.first().unwrap_or(&"food").to_string();
            let reason = format!("Hunger: {:.0} (searching for {})", hunger, food_type);
            Some((utility, reason, food_type))
        }
    }

    /// Calculate utility for a given urgency and distance
    ///
    /// This is a helper function that can be used by custom decision makers.
    ///
    /// # Arguments
    ///
    /// * `urgency` - How urgent the need is (0.0 to 1.0)
    /// * `distance` - Distance to the resource (world units)
    /// * `max_radius` - Maximum search radius
    ///
    /// # Returns
    ///
    /// Calculated utility score (higher is better)
    pub fn calculate_utility(
        &self,
        urgency: f32,
        distance: f32,
        max_radius: f32,
    ) -> f32 {
        let distance_factor = (1.0 - (distance / max_radius)).max(0.0);
        urgency * self.weights.survival + distance_factor * self.weights.efficiency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Agent, AgentIdAllocator};
    use crate::world_query::ResourceLocation;

    // Mock WorldQuery for testing
    struct MockWorldQuery {
        water_sources: Vec<ResourceLocation>,
        food_sources: Vec<ResourceLocation>,
    }

    impl WorldQuery for MockWorldQuery {
        fn get_nearby_agents(&self, _agent: AgentId, _max_count: usize) -> Vec<AgentId> {
            Vec::new()
        }

        fn get_nearby_resources(
            &self,
            _agent: AgentId,
            resource_type: &str,
            _max_radius: f32,
        ) -> Vec<ResourceLocation> {
            match resource_type {
                "water" => self.water_sources.clone(),
                "food" => self.food_sources.clone(),
                _ => Vec::new(),
            }
        }

        fn can_interact(&self, _agent1: AgentId, _agent2: AgentId) -> bool {
            false
        }
    }

    fn create_test_world_with_agent(thirst: f32, hunger: f32, tiredness: f32) -> (World, Entity) {
        let mut world = World::new();
        world.register::<Agent>();
        world.register::<Needs>();
        world.register::<SpeciesComponent>();
        world.insert(AgentIdAllocator::new());

        let mut allocator = world.write_resource::<AgentIdAllocator>();
        let agent_id = allocator.allocate().unwrap();
        drop(allocator);

        let entity = world
            .create_entity()
            .with(Agent { id: agent_id })
            .with(Needs {
                thirst,
                hunger,
                tiredness,
            })
            .build();

        (world, entity)
    }

    #[test]
    fn test_default_configuration() {
        let dm = UtilityMaximizer::default();
        assert_eq!(dm.thresholds.high_thirst, 60.0);
        assert_eq!(dm.weights.survival, 2.0);
        assert_eq!(dm.resource_search_radius, 1000.0);
    }

    #[test]
    fn test_decide_high_thirst_with_nearby_water() {
        let (world, agent) = create_test_world_with_agent(80.0, 20.0, 10.0);

        let world_query = MockWorldQuery {
            water_sources: vec![ResourceLocation::new(100.0, 100.0, 50.0)],
            food_sources: Vec::new(),
        };

        let dm = UtilityMaximizer::default();
        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::SeekItem { item_type, urgency }) => {
                assert_eq!(item_type, "water");
                assert!(urgency > 0.7); // High thirst = high urgency
            }
            _ => panic!("Expected SeekItem intent for water"),
        }
    }

    #[test]
    fn test_decide_high_hunger_seeks_food() {
        let (world, agent) = create_test_world_with_agent(20.0, 70.0, 10.0);

        let world_query = MockWorldQuery {
            water_sources: Vec::new(),
            food_sources: vec![ResourceLocation::new(200.0, 200.0, 100.0)],
        };

        let dm = UtilityMaximizer::default();
        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::SeekItem { item_type, .. }) => {
                assert_eq!(item_type, "food");
            }
            _ => panic!("Expected SeekItem intent for food"),
        }
    }

    #[test]
    fn test_decide_high_tiredness_rests() {
        let (world, agent) = create_test_world_with_agent(20.0, 20.0, 80.0);

        let world_query = MockWorldQuery {
            water_sources: Vec::new(),
            food_sources: Vec::new(),
        };

        let dm = UtilityMaximizer::default();
        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::Rest) => {
                // Success - agent chose to rest
            }
            _ => panic!("Expected Rest intent when tired"),
        }
    }

    #[test]
    fn test_decide_no_urgent_needs_wanders() {
        let (world, agent) = create_test_world_with_agent(30.0, 30.0, 40.0);

        let world_query = MockWorldQuery {
            water_sources: Vec::new(),
            food_sources: Vec::new(),
        };

        let dm = UtilityMaximizer::default();
        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::Wander) => {
                // Success - no urgent needs, so wander
            }
            _ => panic!("Expected Wander intent when no urgent needs"),
        }
    }

    #[test]
    fn test_decide_multiple_needs_selects_highest_utility() {
        let (world, agent) = create_test_world_with_agent(85.0, 65.0, 50.0);
        // Thirst is higher and more urgent

        let world_query = MockWorldQuery {
            water_sources: vec![ResourceLocation::new(100.0, 100.0, 50.0)],
            food_sources: vec![ResourceLocation::new(200.0, 200.0, 100.0)],
        };

        let dm = UtilityMaximizer::default();
        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::SeekItem { item_type, .. }) => {
                // Thirst is higher (85 vs 65), so should seek water
                assert_eq!(item_type, "water");
            }
            _ => panic!("Expected SeekItem intent"),
        }
    }

    #[test]
    fn test_calculate_utility() {
        let dm = UtilityMaximizer::default();

        // High urgency, close distance
        let utility1 = dm.calculate_utility(0.9, 100.0, 1000.0);
        assert!(utility1 > 1.5); // High score

        // Low urgency, far distance
        let utility2 = dm.calculate_utility(0.2, 900.0, 1000.0);
        assert!(utility2 < 0.5); // Low score

        // High urgency beats low urgency regardless of distance
        assert!(utility1 > utility2);
    }

    #[test]
    fn test_evaluate_seek_water_with_source() {
        let dm = UtilityMaximizer::default();
        let agent_id = AgentId(1);

        let world_query = MockWorldQuery {
            water_sources: vec![ResourceLocation::new(100.0, 100.0, 50.0)],
            food_sources: Vec::new(),
        };

        let result = dm.evaluate_seek_water(agent_id, 80.0, &world_query);
        assert!(result.is_some());

        let (utility, reason) = result.unwrap();
        assert!(utility > 1.0); // Should be > survival weight
        assert!(reason.contains("Thirst"));
        assert!(reason.contains("distance"));
    }

    #[test]
    fn test_evaluate_seek_water_no_source() {
        let dm = UtilityMaximizer::default();
        let agent_id = AgentId(1);

        let world_query = MockWorldQuery {
            water_sources: Vec::new(),
            food_sources: Vec::new(),
        };

        let result = dm.evaluate_seek_water(agent_id, 80.0, &world_query);
        assert!(result.is_some());

        let (utility, reason) = result.unwrap();
        // Still has utility (urgency * survival_weight), even without found source
        assert!(utility > 0.0);
        assert!(reason.contains("searching"));
    }

    #[test]
    fn test_custom_configuration() {
        let thresholds = DecisionThresholds {
            high_thirst: 40.0, // Lower threshold = more proactive
            ..Default::default()
        };
        let weights = UtilityWeights {
            survival: 3.0, // Higher weight = prioritize survival more
            ..Default::default()
        };

        let dm = UtilityMaximizer::new(thresholds, weights, 500.0);

        let (world, agent) = create_test_world_with_agent(45.0, 20.0, 10.0);
        // Thirst 45 is above custom threshold (40) but below default (60)

        let world_query = MockWorldQuery {
            water_sources: vec![ResourceLocation::new(100.0, 100.0, 50.0)],
            food_sources: Vec::new(),
        };

        let decision = dm.decide(agent, &world, &world_query);

        match decision {
            DecisionOutput::Intent(Intent::SeekItem { item_type, .. }) => {
                assert_eq!(item_type, "water");
                // Success - custom threshold triggered water seeking
            }
            _ => panic!("Expected water seeking with custom low threshold"),
        }
    }
}
