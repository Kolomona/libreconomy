//! Agent ECS components
use specs::prelude::{Component, VecStorage};
use super::identity::AgentId;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Maximum bound for needs values for performance-friendly f32 usage
pub const MAX_NEEDS: f32 = 100.0;
/// Minimum bound for needs values
pub const MIN_NEEDS: f32 = 0.0;

/// Agent needs component tracking thirst, hunger, and tiredness
///
/// Values are automatically clamped between [`MIN_NEEDS`] and [`MAX_NEEDS`].
///
/// # Example
///
/// ```rust
/// use libreconomy::{Needs, MIN_NEEDS, MAX_NEEDS};
///
/// let needs = Needs::new(50.0, 75.0, 30.0);
/// assert_eq!(needs.thirst, 50.0);
/// assert_eq!(needs.hunger, 75.0);
/// assert_eq!(needs.tiredness, 30.0);
///
/// // Out-of-bounds values are clamped
/// let clamped = Needs::new(-10.0, 200.0, 150.0);
/// assert_eq!(clamped.thirst, MIN_NEEDS);
/// assert_eq!(clamped.hunger, MAX_NEEDS);
/// assert_eq!(clamped.tiredness, MAX_NEEDS);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Needs {
    pub thirst: f32,
    pub hunger: f32,
    pub tiredness: f32,
}

impl Needs {
    /// Creates a new Needs clamped to [MIN_NEEDS, MAX_NEEDS]
    pub fn new(thirst: f32, hunger: f32, tiredness: f32) -> Self {
        fn clamp(v: f32) -> f32 { v.max(MIN_NEEDS).min(MAX_NEEDS) }
        Self {
            thirst: clamp(thirst),
            hunger: clamp(hunger),
            tiredness: clamp(tiredness),
        }
    }

    /// Clamp the current needs in place.
    pub fn clamp(&mut self) {
        self.thirst = self.thirst.max(MIN_NEEDS).min(MAX_NEEDS);
        self.hunger = self.hunger.max(MIN_NEEDS).min(MAX_NEEDS);
        self.tiredness = self.tiredness.max(MIN_NEEDS).min(MAX_NEEDS);
    }
}

impl Component for Needs {
    type Storage = VecStorage<Self>;
}

/// Agent inventory component for storing items
///
/// Maps item IDs (strings) to quantities. Operations are saturating and panic-free.
///
/// # Example
///
/// ```rust
/// use libreconomy::Inventory;
///
/// let mut inv = Inventory::default();
/// 
/// // Add items
/// inv.add("water", 5);
/// assert_eq!(inv.quantity("water"), 5);
///
/// // Remove items (returns amount actually removed)
/// let removed = inv.remove("water", 3);
/// assert_eq!(removed, 3);
/// assert_eq!(inv.quantity("water"), 2);
///
/// // Removing more than available only removes what exists
/// let removed = inv.remove("water", 10);
/// assert_eq!(removed, 2);
/// assert_eq!(inv.quantity("water"), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Inventory {
    pub items: HashMap<String, u32>, // item_id -> quantity
}

impl Inventory {
    /// Get quantity for an item, returns 0 if missing.
    pub fn quantity(&self, item_id: &str) -> u32 {
        *self.items.get(item_id).unwrap_or(&0)
    }

    /// Set absolute quantity (negative not allowed, zero removes entry).
    pub fn set_quantity(&mut self, item_id: &str, quantity: u32) {
        if quantity == 0 {
            self.items.remove(item_id);
        } else {
            self.items.insert(item_id.to_string(), quantity);
        }
    }

    /// Add quantity safely, saturating at u32::MAX.
    pub fn add(&mut self, item_id: &str, delta: u32) {
        if delta == 0 { return; }
        let current = self.quantity(item_id);
        let new_qty = current.saturating_add(delta);
        self.set_quantity(item_id, new_qty);
    }

    /// Remove up to delta; returns removed amount.
    pub fn remove(&mut self, item_id: &str, delta: u32) -> u32 {
        if delta == 0 { return 0; }
        let current = self.quantity(item_id);
        let removed = current.min(delta);
        self.set_quantity(item_id, current.saturating_sub(removed));
        removed
    }
}

impl Component for Inventory {
    type Storage = VecStorage<Self>;
}

/// Agent wallet component for currency management
///
/// Ensures non-negative balance at all times. Negative values are clamped to 0.
///
/// # Example
///
/// ```rust
/// use libreconomy::Wallet;
///
/// let mut wallet = Wallet::new(100.0);
/// assert_eq!(wallet.currency, 100.0);
///
/// // Deposit funds
/// wallet.deposit(50.0);
/// assert_eq!(wallet.currency, 150.0);
///
/// // Withdraw funds (returns amount actually withdrawn)
/// let withdrawn = wallet.withdraw(80.0);
/// assert_eq!(withdrawn, 80.0);
/// assert_eq!(wallet.currency, 70.0);
///
/// // Cannot withdraw more than available
/// let withdrawn = wallet.withdraw(100.0);
/// assert_eq!(withdrawn, 70.0);
/// assert_eq!(wallet.currency, 0.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallet {
    pub currency: f32,
}

impl Wallet {
    /// Create a new wallet with non-negative balance
    pub fn new(currency: f32) -> Self {
        Self { currency: currency.max(0.0) }
    }

    /// Deposit non-negative amount; negative is treated as zero.
    pub fn deposit(&mut self, amount: f32) {
        if amount <= 0.0 { return; }
        self.currency += amount;
    }

    /// Withdraw up to amount, not allowing negative balance; returns withdrawn.
    pub fn withdraw(&mut self, amount: f32) -> f32 {
        if amount <= 0.0 { return 0.0; }
        let withdrawn = self.currency.min(amount);
        self.currency -= withdrawn;
        withdrawn
    }
}

impl Component for Wallet {
    type Storage = VecStorage<Self>;
}

/// Marker/data component designating an entity as an Agent with a unique id
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
}

impl Component for Agent {
    type Storage = VecStorage<Self>;
}

/// Agent skills component.
///
/// Simple key-value skill levels per agent.
///
/// # Example
/// ```rust
/// use libreconomy::Skills;
/// let mut s = Skills::default();
/// s.skills.insert("trading".into(), 2);
/// s.skills.insert("farming".into(), 5);
/// assert_eq!(s.skills.get("trading"), Some(&2));
/// assert_eq!(s.skills.get("farming"), Some(&5));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Skills {
    pub skills: HashMap<String, u32>, // skill_id -> level
}
impl Component for Skills {
    type Storage = VecStorage<Self>;
}

/// Agent knowledge component.
///
/// Stores observed prices and known trade partners.
///
/// # Example
/// ```rust
/// use libreconomy::{Knowledge, LearningSystem};
/// let mut k = Knowledge::default();
/// LearningSystem::update(&mut k, "water", 3.5);
/// LearningSystem::update(&mut k, "water", 4.0);
/// assert_eq!(k.known_prices.get("water"), Some(&4.0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Knowledge {
    pub known_prices: std::collections::HashMap<String, f32>,
    pub trade_partners: Vec<String>,
}
impl Component for Knowledge {
    type Storage = VecStorage<Self>;
}

/// Agent employment component.
///
/// Tracks job status, employer, and subordinates.
///
/// # Example
/// ```rust
/// use libreconomy::Employment;
/// let mut e = Employment::default();
/// e.job_status = Some("employed".into());
/// e.employer = Some("Acme Inc".into());
/// e.employees.push("WorkerA".into());
/// assert_eq!(e.job_status.as_deref(), Some("employed"));
/// assert_eq!(e.employer.as_deref(), Some("Acme Inc"));
/// assert_eq!(e.employees.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Employment {
    pub job_status: Option<String>,
    pub employer: Option<String>,
    pub employees: Vec<String>,
}
impl Component for Employment {
    type Storage = VecStorage<Self>;
}

/// Utility function type for preferences.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UtilityFunctionType {
    Linear,
    Exponential,
    Custom(String), // Placeholder for custom logic
}

/// Agent preferences component.
///
/// # Example
/// ```rust
/// use libreconomy::{Preferences, UtilityFunctionType};
/// let p = Preferences { utility_function: UtilityFunctionType::Linear, risk_tolerance: 0.3 };
/// assert!(matches!(p.utility_function, UtilityFunctionType::Linear));
/// assert_eq!(p.risk_tolerance, 0.3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub utility_function: UtilityFunctionType,
    pub risk_tolerance: f32,
}

impl Component for Preferences {
    type Storage = VecStorage<Self>;
}

/// Resource source component for entities that produce items.
///
/// Used to mark entities as resource providers (e.g., water sources, grass patches).
/// Supports regeneration and stock tracking.
///
/// # Example
/// ```rust
/// use libreconomy::ResourceSource;
///
/// let grass_patch = ResourceSource {
///     resource_type: "plant".to_string(),
///     item_produced: "grass".to_string(),
///     regeneration_rate: 0.1,
///     current_stock: 100,
/// };
///
/// assert_eq!(grass_patch.item_produced, "grass");
/// assert_eq!(grass_patch.current_stock, 100);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSource {
    /// Type of resource (e.g., "plant", "water", "mineral")
    pub resource_type: String,
    /// The item ID this source produces
    pub item_produced: String,
    /// How much stock regenerates per tick (0.0 = non-renewable)
    pub regeneration_rate: f32,
    /// Current available quantity
    pub current_stock: u32,
}

impl ResourceSource {
    /// Create a new resource source
    pub fn new(
        resource_type: String,
        item_produced: String,
        regeneration_rate: f32,
        current_stock: u32,
    ) -> Self {
        Self {
            resource_type,
            item_produced,
            regeneration_rate,
            current_stock,
        }
    }

    /// Harvest from this resource source, returning amount actually harvested
    pub fn harvest(&mut self, amount: u32) -> u32 {
        let harvested = self.current_stock.min(amount);
        self.current_stock = self.current_stock.saturating_sub(harvested);
        harvested
    }

    /// Regenerate resource stock
    pub fn regenerate(&mut self) {
        if self.regeneration_rate > 0.0 {
            let regen_amount = self.regeneration_rate as u32;
            self.current_stock = self.current_stock.saturating_add(regen_amount);
        }
    }

    /// Check if resource is available
    pub fn is_available(&self) -> bool {
        self.current_stock > 0
    }
}

impl Component for ResourceSource {
    type Storage = VecStorage<Self>;
}

/// Species type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    Human,
    Rabbit,
    Custom(u32),
}

/// Diet type defining what a species can eat
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DietType {
    /// Herbivore - eats plants only
    Herbivore {
        preferred_plants: Vec<String>,
    },
    /// Carnivore - eats prey only
    Carnivore {
        preferred_prey: Vec<Species>,
    },
    /// Omnivore - eats both plants and prey
    Omnivore {
        plants: Vec<String>,
        prey: Vec<Species>,
    },
}

impl DietType {
    /// Check if this diet can consume a specific plant item
    pub fn can_eat_plant(&self, plant_id: &str) -> bool {
        match self {
            DietType::Herbivore { preferred_plants } => {
                preferred_plants.is_empty() || preferred_plants.contains(&plant_id.to_string())
            }
            DietType::Omnivore { plants, .. } => {
                plants.is_empty() || plants.contains(&plant_id.to_string())
            }
            DietType::Carnivore { .. } => false,
        }
    }

    /// Check if this diet can hunt a specific species
    pub fn can_hunt(&self, prey_species: Species) -> bool {
        match self {
            DietType::Carnivore { preferred_prey } => {
                preferred_prey.is_empty() || preferred_prey.contains(&prey_species)
            }
            DietType::Omnivore { prey, .. } => {
                prey.is_empty() || prey.contains(&prey_species)
            }
            DietType::Herbivore { .. } => false,
        }
    }
}

/// Species component for agents
///
/// Defines the species and dietary preferences of an agent.
///
/// # Example
/// ```rust
/// use libreconomy::{SpeciesComponent, Species, DietType};
///
/// // Create a rabbit (herbivore)
/// let rabbit = SpeciesComponent {
///     species: Species::Rabbit,
///     diet: DietType::Herbivore {
///         preferred_plants: vec!["grass".to_string()],
///     },
/// };
///
/// assert!(rabbit.diet.can_eat_plant("grass"));
/// assert!(!rabbit.diet.can_hunt(Species::Human));
///
/// // Create a human (omnivore)
/// let human = SpeciesComponent {
///     species: Species::Human,
///     diet: DietType::Omnivore {
///         plants: vec!["grass".to_string(), "fruit".to_string()],
///         prey: vec![Species::Rabbit],
///     },
/// };
///
/// assert!(human.diet.can_eat_plant("grass"));
/// assert!(human.diet.can_hunt(Species::Rabbit));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesComponent {
    pub species: Species,
    pub diet: DietType,
}

impl SpeciesComponent {
    /// Create a default rabbit species (herbivore eating grass)
    pub fn rabbit() -> Self {
        Self {
            species: Species::Rabbit,
            diet: DietType::Herbivore {
                preferred_plants: vec!["grass".to_string()],
            },
        }
    }

    /// Create a default human species (omnivore)
    pub fn human() -> Self {
        Self {
            species: Species::Human,
            diet: DietType::Omnivore {
                plants: vec!["grass".to_string(), "food".to_string()],
                prey: vec![Species::Rabbit],
            },
        }
    }

    /// Create a custom species with specified diet
    pub fn custom(id: u32, diet: DietType) -> Self {
        Self {
            species: Species::Custom(id),
            diet,
        }
    }
}

impl Component for SpeciesComponent {
    type Storage = VecStorage<Self>;
}

/// Reputation view of a single agent using Beta distribution
///
/// Uses Beta(alpha, beta) to model reputation based on positive/negative interactions.
/// Score is computed as alpha / (alpha + beta), representing expected trustworthiness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReputationView {
    /// Alpha parameter (positive interactions)
    pub alpha: f32,
    /// Beta parameter (negative interactions)
    pub beta: f32,
    /// Last interaction timestamp (for decay)
    pub last_interaction_tick: u64,
    /// Total number of interactions
    pub interaction_count: u32,
}

impl ReputationView {
    /// Create a new reputation view with uniform prior (1, 1)
    pub fn new() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            last_interaction_tick: 0,
            interaction_count: 0,
        }
    }

    /// Create a reputation view with custom prior
    pub fn with_prior(alpha: f32, beta: f32) -> Self {
        Self {
            alpha,
            beta,
            last_interaction_tick: 0,
            interaction_count: 0,
        }
    }

    /// Get the reputation score (expected value of Beta distribution)
    ///
    /// Returns a value in [0, 1] where 1.0 = fully trusted, 0.0 = fully distrusted
    pub fn score(&self) -> f32 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Get reputation score with temporal decay
    ///
    /// Applies exponential decay based on time since last interaction.
    /// Older reputation becomes less certain (regresses toward prior).
    ///
    /// # Arguments
    /// * `current_tick` - Current simulation tick
    /// * `decay_rate` - Decay rate per tick (e.g., 0.001 = 0.1% decay per tick)
    pub fn score_with_decay(&self, current_tick: u64, decay_rate: f32) -> f32 {
        let ticks_since_interaction = current_tick.saturating_sub(self.last_interaction_tick);
        let decay_factor = (-decay_rate * ticks_since_interaction as f32).exp();

        // Decay toward neutral score (0.5)
        let current_score = self.score();
        0.5 + (current_score - 0.5) * decay_factor
    }

    /// Update reputation based on interaction outcome
    ///
    /// # Arguments
    /// * `outcome_weight` - Positive for good interaction, negative for bad
    /// * `current_tick` - Current simulation tick
    pub fn update(&mut self, outcome_weight: f32, current_tick: u64) {
        if outcome_weight > 0.0 {
            self.alpha += outcome_weight;
        } else if outcome_weight < 0.0 {
            self.beta += outcome_weight.abs();
        }

        self.last_interaction_tick = current_tick;
        self.interaction_count += 1;
    }

    /// Get confidence in this reputation (total evidence)
    ///
    /// Higher values indicate more interactions, thus more confident estimate
    pub fn confidence(&self) -> f32 {
        self.alpha + self.beta
    }
}

impl Default for ReputationView {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent reputation knowledge component
///
/// Tracks first-hand reputation observations of other agents.
///
/// # Example
/// ```rust
/// use libreconomy::{ReputationKnowledge, ReputationView, AgentId};
/// use std::collections::HashMap;
///
/// let mut rep = ReputationKnowledge::default();
///
/// // Update reputation after a positive interaction
/// let partner = AgentId(42);
/// rep.update_reputation(partner, 1.0, 100);
///
/// // Check reputation score
/// let score = rep.get_score(partner);
/// assert!(score > 0.5); // Positive interaction increases score above neutral
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReputationKnowledge {
    /// First-hand reputation views of known agents
    pub first_hand: HashMap<AgentId, ReputationView>,
    /// Baseline trust level for unknown agents (0.0 = distrust, 1.0 = trust)
    pub trust_level: f32,
}

impl ReputationKnowledge {
    /// Create a new reputation knowledge with default trust level (0.5)
    pub fn new() -> Self {
        Self {
            first_hand: HashMap::new(),
            trust_level: 0.5,
        }
    }

    /// Create a reputation knowledge with custom default trust level
    pub fn with_trust_level(trust_level: f32) -> Self {
        Self {
            first_hand: HashMap::new(),
            trust_level: trust_level.clamp(0.0, 1.0),
        }
    }

    /// Get reputation score for an agent
    ///
    /// Returns the score if known, otherwise returns the default trust level
    pub fn get_score(&self, agent: AgentId) -> f32 {
        self.first_hand
            .get(&agent)
            .map(|view| view.score())
            .unwrap_or(self.trust_level)
    }

    /// Get reputation score with decay for an agent
    pub fn get_score_with_decay(&self, agent: AgentId, current_tick: u64, decay_rate: f32) -> f32 {
        self.first_hand
            .get(&agent)
            .map(|view| view.score_with_decay(current_tick, decay_rate))
            .unwrap_or(self.trust_level)
    }

    /// Update reputation based on interaction outcome
    ///
    /// # Arguments
    /// * `agent` - The agent being evaluated
    /// * `outcome_weight` - Positive for good interaction, negative for bad
    /// * `current_tick` - Current simulation tick
    pub fn update_reputation(&mut self, agent: AgentId, outcome_weight: f32, current_tick: u64) {
        self.first_hand
            .entry(agent)
            .or_insert_with(ReputationView::new)
            .update(outcome_weight, current_tick);
    }

    /// Check if an agent is trusted (score above threshold)
    pub fn is_trusted(&self, agent: AgentId, threshold: f32) -> bool {
        self.get_score(agent) >= threshold
    }

    /// Get the most trusted agents
    ///
    /// Returns up to `max_count` agents sorted by reputation score (highest first)
    pub fn get_most_trusted(&self, max_count: usize) -> Vec<(AgentId, f32)> {
        let mut scores: Vec<(AgentId, f32)> = self
            .first_hand
            .iter()
            .map(|(id, view)| (*id, view.score()))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(max_count);
        scores
    }
}

impl Component for ReputationKnowledge {
    type Storage = VecStorage<Self>;
}

pub struct NeedDecaySystem;
impl NeedDecaySystem {
    pub fn tick(needs: &mut Needs) {
        needs.thirst += 0.01;
        needs.hunger += 0.01;
    }
}

pub struct LearningSystem;
impl LearningSystem {
    pub fn update(knowledge: &mut Knowledge, item: &str, price: f32) {
        knowledge.known_prices.insert(item.to_string(), price);
    }
}

pub struct NegotiationSystem;
impl NegotiationSystem {
    pub fn negotiate() -> bool {
        true // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_needs_creation() {
        let needs = Needs { thirst: 1.0, hunger: 2.0, tiredness: 3.0 };
        assert_eq!(needs.thirst, 1.0);
        assert_eq!(needs.hunger, 2.0);
        assert_eq!(needs.tiredness, 3.0);
    }

    #[test]
    fn test_needs_boundaries() {
        let needs = Needs { thirst: 0.0, hunger: 100.0, tiredness: 50.0 };
        assert!(needs.thirst >= 0.0 && needs.thirst <= 100.0);
        assert!(needs.hunger >= 0.0 && needs.hunger <= 100.0);
        assert!(needs.tiredness >= 0.0 && needs.tiredness <= 100.0);
    }

    #[test]
    fn test_resource_source_creation() {
        let grass = ResourceSource::new(
            "plant".to_string(),
            "grass".to_string(),
            0.1,
            100,
        );

        assert_eq!(grass.resource_type, "plant");
        assert_eq!(grass.item_produced, "grass");
        assert_eq!(grass.regeneration_rate, 0.1);
        assert_eq!(grass.current_stock, 100);
        assert!(grass.is_available());
    }

    #[test]
    fn test_resource_source_harvest() {
        let mut grass = ResourceSource::new(
            "plant".to_string(),
            "grass".to_string(),
            0.1,
            100,
        );

        // Harvest some
        let harvested = grass.harvest(30);
        assert_eq!(harvested, 30);
        assert_eq!(grass.current_stock, 70);

        // Harvest more than available
        let harvested = grass.harvest(100);
        assert_eq!(harvested, 70);
        assert_eq!(grass.current_stock, 0);
        assert!(!grass.is_available());
    }

    #[test]
    fn test_resource_source_regenerate() {
        let mut grass = ResourceSource::new(
            "plant".to_string(),
            "grass".to_string(),
            5.0,
            100,
        );

        grass.harvest(50);
        assert_eq!(grass.current_stock, 50);

        grass.regenerate();
        assert_eq!(grass.current_stock, 55);

        grass.regenerate();
        assert_eq!(grass.current_stock, 60);
    }

    #[test]
    fn test_resource_source_non_renewable() {
        let mut ore = ResourceSource::new(
            "mineral".to_string(),
            "iron_ore".to_string(),
            0.0,
            50,
        );

        ore.harvest(20);
        assert_eq!(ore.current_stock, 30);

        ore.regenerate();
        assert_eq!(ore.current_stock, 30); // No regeneration
    }

    #[test]
    fn test_species_rabbit() {
        let rabbit = SpeciesComponent::rabbit();
        assert_eq!(rabbit.species, Species::Rabbit);
        assert!(rabbit.diet.can_eat_plant("grass"));
        assert!(!rabbit.diet.can_hunt(Species::Human));
        assert!(!rabbit.diet.can_hunt(Species::Rabbit));
    }

    #[test]
    fn test_species_human() {
        let human = SpeciesComponent::human();
        assert_eq!(human.species, Species::Human);
        assert!(human.diet.can_eat_plant("grass"));
        assert!(human.diet.can_eat_plant("food"));
        assert!(human.diet.can_hunt(Species::Rabbit));
        assert!(!human.diet.can_hunt(Species::Human));
    }

    #[test]
    fn test_diet_herbivore() {
        let diet = DietType::Herbivore {
            preferred_plants: vec!["grass".to_string(), "leaves".to_string()],
        };

        assert!(diet.can_eat_plant("grass"));
        assert!(diet.can_eat_plant("leaves"));
        assert!(!diet.can_eat_plant("fruit"));
        assert!(!diet.can_hunt(Species::Rabbit));
        assert!(!diet.can_hunt(Species::Human));
    }

    #[test]
    fn test_diet_carnivore() {
        let diet = DietType::Carnivore {
            preferred_prey: vec![Species::Rabbit],
        };

        assert!(!diet.can_eat_plant("grass"));
        assert!(!diet.can_eat_plant("anything"));
        assert!(diet.can_hunt(Species::Rabbit));
        assert!(!diet.can_hunt(Species::Human));
    }

    #[test]
    fn test_diet_omnivore() {
        let diet = DietType::Omnivore {
            plants: vec!["grass".to_string()],
            prey: vec![Species::Rabbit],
        };

        assert!(diet.can_eat_plant("grass"));
        assert!(!diet.can_eat_plant("fruit"));
        assert!(diet.can_hunt(Species::Rabbit));
        assert!(!diet.can_hunt(Species::Human));
    }

    #[test]
    fn test_diet_empty_preferences_accepts_all() {
        let herbivore = DietType::Herbivore {
            preferred_plants: vec![],
        };
        assert!(herbivore.can_eat_plant("anything"));

        let carnivore = DietType::Carnivore {
            preferred_prey: vec![],
        };
        assert!(carnivore.can_hunt(Species::Rabbit));
        assert!(carnivore.can_hunt(Species::Human));
    }

    #[test]
    fn test_reputation_view_creation() {
        let view = ReputationView::new();
        assert_eq!(view.alpha, 1.0);
        assert_eq!(view.beta, 1.0);
        assert_eq!(view.score(), 0.5); // Uniform prior
        assert_eq!(view.interaction_count, 0);
    }

    #[test]
    fn test_reputation_view_positive_update() {
        let mut view = ReputationView::new();
        view.update(1.0, 100);

        assert_eq!(view.alpha, 2.0);
        assert_eq!(view.beta, 1.0);
        assert!(view.score() > 0.5); // Increased trust
        assert_eq!(view.interaction_count, 1);
        assert_eq!(view.last_interaction_tick, 100);
    }

    #[test]
    fn test_reputation_view_negative_update() {
        let mut view = ReputationView::new();
        view.update(-1.0, 100);

        assert_eq!(view.alpha, 1.0);
        assert_eq!(view.beta, 2.0);
        assert!(view.score() < 0.5); // Decreased trust
        assert_eq!(view.interaction_count, 1);
    }

    #[test]
    fn test_reputation_view_score_bounds() {
        let mut view = ReputationView::new();

        // Many positive interactions
        for _ in 0..10 {
            view.update(1.0, 100);
        }
        assert!(view.score() >= 0.0 && view.score() <= 1.0);
        assert!(view.score() > 0.9); // Very high trust

        // Many negative interactions
        for _ in 0..20 {
            view.update(-1.0, 200);
        }
        assert!(view.score() >= 0.0 && view.score() <= 1.0);
        assert!(view.score() < 0.5); // Now distrusted
    }

    #[test]
    fn test_reputation_view_decay() {
        let mut view = ReputationView::new();
        view.update(10.0, 100); // Strong positive
        let score_fresh = view.score();

        // Score should decay toward 0.5 over time
        let score_decayed = view.score_with_decay(1100, 0.001);
        assert!(score_decayed < score_fresh);
        assert!(score_decayed > 0.5);
    }

    #[test]
    fn test_reputation_knowledge_creation() {
        let rep = ReputationKnowledge::new();
        assert_eq!(rep.trust_level, 0.5);
        assert_eq!(rep.first_hand.len(), 0);
    }

    #[test]
    fn test_reputation_knowledge_unknown_agent() {
        let rep = ReputationKnowledge::with_trust_level(0.7);
        let unknown_agent = AgentId(999);

        let score = rep.get_score(unknown_agent);
        assert_eq!(score, 0.7); // Returns default trust level
    }

    #[test]
    fn test_reputation_knowledge_update() {
        let mut rep = ReputationKnowledge::new();
        let agent = AgentId(42);

        rep.update_reputation(agent, 1.0, 100);
        let score = rep.get_score(agent);
        assert!(score > 0.5);

        rep.update_reputation(agent, -2.0, 200);
        let new_score = rep.get_score(agent);
        assert!(new_score < score); // Negative interaction decreases score
    }

    #[test]
    fn test_reputation_knowledge_is_trusted() {
        let mut rep = ReputationKnowledge::new();
        let agent = AgentId(42);

        assert!(!rep.is_trusted(agent, 0.8)); // Unknown, default is 0.5

        for _ in 0..5 {
            rep.update_reputation(agent, 1.0, 100);
        }

        assert!(rep.is_trusted(agent, 0.8)); // Now trusted
    }

    #[test]
    fn test_reputation_knowledge_most_trusted() {
        let mut rep = ReputationKnowledge::new();

        let agent1 = AgentId(1);
        let agent2 = AgentId(2);
        let agent3 = AgentId(3);

        // Agent 1: very trusted
        for _ in 0..5 {
            rep.update_reputation(agent1, 1.0, 100);
        }

        // Agent 2: neutral
        rep.update_reputation(agent2, 1.0, 100);
        rep.update_reputation(agent2, -1.0, 200);

        // Agent 3: distrusted
        for _ in 0..3 {
            rep.update_reputation(agent3, -1.0, 100);
        }

        let most_trusted = rep.get_most_trusted(2);
        assert_eq!(most_trusted.len(), 2);
        assert_eq!(most_trusted[0].0, agent1); // Highest trust
        assert!(most_trusted[0].1 > most_trusted[1].1);
    }
}
