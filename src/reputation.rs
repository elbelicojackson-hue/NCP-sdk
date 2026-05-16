//! ELO-based Dynamic Reputation System
//!
//! Each agent has a rating that changes based on attack/defense outcomes.
//! Higher rating = higher vote weight in consensus decisions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

const K_BASE: f64 = 32.0;
const INITIAL_RATING: f64 = 1000.0;
const SURVIVE_BONUS: f64 = 5.0;

/// ELO reputation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEngine {
    ratings: HashMap<String, f64>,
}

impl ReputationEngine {
    pub fn new() -> Self {
        Self {
            ratings: HashMap::new(),
        }
    }

    /// Register an agent with initial rating
    pub fn register(&mut self, agent_id: &str) {
        self.ratings
            .entry(agent_id.to_string())
            .or_insert(INITIAL_RATING);
    }

    /// Get an agent's current rating
    pub fn rating(&self, agent_id: &str) -> f64 {
        *self.ratings.get(agent_id).unwrap_or(&INITIAL_RATING)
    }

    /// Record attack result and update both parties' ratings
    ///
    /// ELO formula:
    ///   E_a = 1 / (1 + 10^((R_d - R_a) / 400))
    ///   R_a' = R_a + K × (actual - E_a)
    pub fn record_attack(&mut self, attacker_id: &str, defender_id: &str, success: bool) {
        self.register(attacker_id);
        self.register(defender_id);

        let r_a = self.ratings[attacker_id];
        let r_d = self.ratings[defender_id];

        // Expected scores
        let e_a = 1.0 / (1.0 + 10_f64.powf((r_d - r_a) / 400.0));
        let e_d = 1.0 - e_a;

        // Actual scores
        let (actual_a, actual_d) = if success { (1.0, 0.0) } else { (0.0, 1.0) };

        // Update
        *self.ratings.get_mut(attacker_id).unwrap() += K_BASE * (actual_a - e_a);
        *self.ratings.get_mut(defender_id).unwrap() += K_BASE * (actual_d - e_d);
    }

    /// Award survival bonus (chain survived a round without being broken)
    pub fn record_survive(&mut self, agent_id: &str) {
        self.register(agent_id);
        *self.ratings.get_mut(agent_id).unwrap() += SURVIVE_BONUS;
    }

    /// Get vote weight for an agent
    ///
    /// Mapping: R=800 → 0.5, R=1000 → 1.0, R=1200 → 1.5, R=1400 → 2.0
    pub fn vote_weight(&self, agent_id: &str) -> f64 {
        let rating = self.rating(agent_id);
        let weight = 0.5 + (rating - 800.0) / 400.0;
        weight.clamp(0.5, 2.0)
    }

    /// Get ranking (sorted by rating descending)
    pub fn ranking(&self) -> Vec<(&str, f64)> {
        let mut ranked: Vec<(&str, f64)> = self
            .ratings
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_rating() {
        let engine = ReputationEngine::new();
        assert_eq!(engine.rating("unknown"), INITIAL_RATING);
    }

    #[test]
    fn test_attack_success_increases_attacker() {
        let mut engine = ReputationEngine::new();
        engine.register("attacker");
        engine.register("defender");

        let before = engine.rating("attacker");
        engine.record_attack("attacker", "defender", true);
        assert!(engine.rating("attacker") > before);
        assert!(engine.rating("defender") < INITIAL_RATING);
    }

    #[test]
    fn test_vote_weight_bounds() {
        let mut engine = ReputationEngine::new();
        engine.ratings.insert("low".to_string(), 500.0);
        engine.ratings.insert("high".to_string(), 2000.0);

        assert_eq!(engine.vote_weight("low"), 0.5);
        assert_eq!(engine.vote_weight("high"), 2.0);
    }
}
