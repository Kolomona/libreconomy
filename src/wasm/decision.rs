//! WASM decision system bindings
//!
//! Provides JavaScript-friendly interface to the decision-making system.

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;
use specs::WorldExt;

use crate::{
    UtilityMaximizer, DecisionThresholds, UtilityWeights,
    WorldQuery, ResourceLocation, AgentId, DecisionOutput, Intent,
};
use super::world::WasmWorld;

/// JavaScript callback interface for WorldQuery
///
/// This allows JavaScript to provide spatial information to the decision system.
#[wasm_bindgen]
extern "C" {
    /// JavaScript type for WorldQuery callbacks
    pub type JsWorldQuery;

    #[wasm_bindgen(structural, method, js_name = getNearbyAgents)]
    fn get_nearby_agents(this: &JsWorldQuery, agent_id: u32, max_count: usize) -> JsValue;

    #[wasm_bindgen(structural, method, js_name = getNearbyResources)]
    fn get_nearby_resources(
        this: &JsWorldQuery,
        agent_id: u32,
        resource_type: &str,
        max_radius: f32,
    ) -> JsValue;

    #[wasm_bindgen(structural, method, js_name = canInteract)]
    fn can_interact(this: &JsWorldQuery, agent1_id: u32, agent2_id: u32) -> bool;
}

/// Adapter that implements WorldQuery for JavaScript callbacks
struct JsWorldQueryAdapter<'a> {
    js_query: &'a JsWorldQuery,
}

impl<'a> WorldQuery for JsWorldQueryAdapter<'a> {
    fn get_nearby_agents(&self, agent: AgentId, max_count: usize) -> Vec<AgentId> {
        let js_result = self.js_query.get_nearby_agents(agent.0 as u32, max_count);

        // Convert JsValue array to Vec<AgentId>
        serde_wasm_bindgen::from_value::<Vec<u32>>(js_result)
            .unwrap_or_default()
            .into_iter()
            .map(|id| AgentId(id as u64))
            .collect()
    }

    fn get_nearby_resources(
        &self,
        agent: AgentId,
        resource_type: &str,
        max_radius: f32,
    ) -> Vec<ResourceLocation> {
        let js_result = self
            .js_query
            .get_nearby_resources(agent.0 as u32, resource_type, max_radius);

        // Convert JsValue array to Vec<ResourceLocation>
        serde_wasm_bindgen::from_value(js_result).unwrap_or_default()
    }

    fn can_interact(&self, agent1: AgentId, agent2: AgentId) -> bool {
        self.js_query
            .can_interact(agent1.0 as u32, agent2.0 as u32)
    }
}

/// WASM wrapper for decision-making system
///
/// Create a decision maker and use it to generate agent decisions.
/// Requires a WorldQuery implementation to provide spatial information.
#[wasm_bindgen]
pub struct WasmDecisionMaker {
    inner: UtilityMaximizer,
}

#[wasm_bindgen]
impl WasmDecisionMaker {
    /// Create a new decision maker with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: UtilityMaximizer::default(),
        }
    }

    /// Create a decision maker with custom configuration
    #[wasm_bindgen(js_name = withConfig)]
    pub fn with_config(
        critical_thirst: f32,
        high_thirst: f32,
        critical_hunger: f32,
        high_hunger: f32,
        critical_tiredness: f32,
        high_tiredness: f32,
        survival_weight: f32,
        comfort_weight: f32,
        efficiency_weight: f32,
        search_radius: f32,
    ) -> Self {
        let thresholds = DecisionThresholds {
            critical_thirst,
            high_thirst,
            critical_hunger,
            high_hunger,
            critical_tiredness,
            high_tiredness,
        };

        let weights = UtilityWeights {
            survival: survival_weight,
            comfort: comfort_weight,
            efficiency: efficiency_weight,
        };

        Self {
            inner: UtilityMaximizer {
                thresholds,
                weights,
                resource_search_radius: search_radius,
            },
        }
    }

    /// Make a decision for an agent
    ///
    /// # Arguments
    /// * `world` - The WASM world containing the agent
    /// * `entity_id` - The agent's entity ID
    /// * `world_query` - JavaScript object implementing WorldQuery interface
    ///
    /// # Returns
    /// JSON-serialized DecisionOutput
    pub fn decide(
        &self,
        world: &WasmWorld,
        entity_id: u32,
        world_query: &JsWorldQuery,
    ) -> JsValue {
        // Get the entity
        let entity = world.get_world().entities().entity(entity_id);

        // Create adapter for JavaScript WorldQuery
        let query_adapter = JsWorldQueryAdapter { js_query: world_query };

        // Make decision
        let decision = self.inner.decide(entity, world.get_world(), &query_adapter);

        // Serialize to JsValue
        serde_wasm_bindgen::to_value(&decision).unwrap_or(JsValue::NULL)
    }

    /// Make a decision for libreterra (returns compatible format)
    ///
    /// Returns a decision result compatible with libreconomy-stub.js format.
    /// This provides a more JavaScript-friendly interface for libreterra.
    ///
    /// # Arguments
    /// * `world` - The WASM world containing the agent
    /// * `entity_id` - The agent's entity ID
    /// * `world_query` - JavaScript object implementing WorldQuery interface
    ///
    /// # Returns
    /// JsDecisionResult with intent type, targets, utility, and reason
    pub fn decide_libreterra(
        &self,
        world: &WasmWorld,
        entity_id: u32,
        world_query: &JsWorldQuery,
    ) -> JsDecisionResult {
        let entity = world.get_world().entities().entity(entity_id);
        let query_adapter = JsWorldQueryAdapter { js_query: world_query };
        let decision = self.inner.decide(entity, world.get_world(), &query_adapter);

        // Convert DecisionOutput to JsDecisionResult
        match decision {
            DecisionOutput::Intent(intent) => match intent {
                Intent::SeekItem { item_type, urgency } => {
                    let intent_type = if item_type == "water" {
                        "SEEK_WATER".to_string()
                    } else {
                        "SEEK_FOOD".to_string()
                    };

                    JsDecisionResult {
                        intent_type,
                        target_x: 0.0,
                        target_y: 0.0,
                        has_target: false,
                        target_entity: 0,
                        has_target_entity: false,
                        utility: urgency,
                        reason: format!("Seeking {} (urgency: {:.2})", item_type, urgency),
                    }
                }
                Intent::Rest => JsDecisionResult {
                    intent_type: "REST".to_string(),
                    target_x: 0.0,
                    target_y: 0.0,
                    has_target: false,
                    target_entity: 0,
                    has_target_entity: false,
                    utility: 0.5,
                    reason: "Resting to recover".to_string(),
                },
                Intent::Wander => JsDecisionResult {
                    intent_type: "WANDER".to_string(),
                    target_x: 0.0,
                    target_y: 0.0,
                    has_target: false,
                    target_entity: 0,
                    has_target_entity: false,
                    utility: 0.1,
                    reason: "Idle exploration".to_string(),
                },
                _ => JsDecisionResult {
                    intent_type: "WANDER".to_string(),
                    target_x: 0.0,
                    target_y: 0.0,
                    has_target: false,
                    target_entity: 0,
                    has_target_entity: false,
                    utility: 0.1,
                    reason: "Unsupported intent type".to_string(),
                },
            },
            _ => JsDecisionResult {
                intent_type: "WANDER".to_string(),
                target_x: 0.0,
                target_y: 0.0,
                has_target: false,
                target_entity: 0,
                has_target_entity: false,
                utility: 0.1,
                reason: "No intent available".to_string(),
            },
        }
    }
}

/// JavaScript-friendly decision result
///
/// Converts Rust DecisionOutput to a format matching libreconomy-stub.js
#[wasm_bindgen]
pub struct JsDecisionResult {
    intent_type: String,
    target_x: f32,
    target_y: f32,
    has_target: bool,
    target_entity: u32,
    has_target_entity: bool,
    utility: f32,
    reason: String,
}

#[wasm_bindgen]
impl JsDecisionResult {
    #[wasm_bindgen(getter)]
    pub fn intent_type(&self) -> String {
        self.intent_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn target_x(&self) -> f32 {
        self.target_x
    }

    #[wasm_bindgen(getter)]
    pub fn target_y(&self) -> f32 {
        self.target_y
    }

    #[wasm_bindgen(getter)]
    pub fn has_target(&self) -> bool {
        self.has_target
    }

    #[wasm_bindgen(getter)]
    pub fn target_entity(&self) -> u32 {
        self.target_entity
    }

    #[wasm_bindgen(getter)]
    pub fn has_target_entity(&self) -> bool {
        self.has_target_entity
    }

    #[wasm_bindgen(getter)]
    pub fn utility(&self) -> f32 {
        self.utility
    }

    #[wasm_bindgen(getter)]
    pub fn reason(&self) -> String {
        self.reason.clone()
    }
}

// Note: WasmWorld needs a method to access the inner World
// This will be added to world.rs
