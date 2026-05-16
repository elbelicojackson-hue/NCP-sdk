//! UCB1 Attack Scheduling
//!
//! Models attack target selection as a multi-armed bandit.
//! Balances exploitation (attack steps that collapse easily)
//! with exploration (attack steps never verified).

use std::collections::HashMap;

/// Statistics for a single step
#[derive(Debug, Clone)]
pub struct StepStats {
    pub agent_id: String,
    pub step_index: u32,
    pub attack_count: u32,
    pub collapse_count: u32,
    pub defend_count: u32,
    pub last_attack_round: i32,
    pub defense_reasons: Vec<String>,
}

/// UCB1-based attack scheduler
pub struct AttackScheduler {
    stats: HashMap<String, StepStats>, // key: "agent_id.step_index"
    total_attacks: u32,
    exploration_constant: f64, // C = sqrt(2) by default
}

impl AttackScheduler {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
            total_attacks: 0,
            exploration_constant: std::f64::consts::SQRT_2,
        }
    }

    /// Register a step as attackable
    pub fn register_step(&mut self, agent_id: &str, step_index: u32) {
        let key = format!("{}.{}", agent_id, step_index);
        self.stats.entry(key).or_insert(StepStats {
            agent_id: agent_id.to_string(),
            step_index,
            attack_count: 0,
            collapse_count: 0,
            defend_count: 0,
            last_attack_round: -1,
            defense_reasons: Vec::new(),
        });
    }

    /// Register all steps of a chain
    pub fn register_chain(&mut self, agent_id: &str, step_count: u32) {
        for i in 1..=step_count {
            self.register_step(agent_id, i);
        }
    }

    /// Record an attack result
    pub fn record_attack(
        &mut self,
        agent_id: &str,
        step_index: u32,
        success: bool,
        round: u32,
        defense_reason: Option<&str>,
    ) {
        let key = format!("{}.{}", agent_id, step_index);
        let stat = self.stats.entry(key).or_insert(StepStats {
            agent_id: agent_id.to_string(),
            step_index,
            attack_count: 0,
            collapse_count: 0,
            defend_count: 0,
            last_attack_round: -1,
            defense_reasons: Vec::new(),
        });

        stat.attack_count += 1;
        stat.last_attack_round = round as i32;
        self.total_attacks += 1;

        if success {
            stat.collapse_count += 1;
        } else {
            stat.defend_count += 1;
            if let Some(reason) = defense_reason {
                if !stat.defense_reasons.contains(&reason.to_string()) {
                    stat.defense_reasons.push(reason.to_string());
                }
            }
        }
    }

    /// Compute UCB1 score for a step
    ///
    /// UCB(i) = exploitation + C × sqrt(ln(N) / n_i) + recency_bonus
    ///
    /// - exploitation = collapse_rate (higher = easier to break)
    /// - exploration = sqrt(ln(N) / n_i) (higher = less explored)
    /// - recency = 0.1 × ln(1 + rounds_since_last_attack)
    pub fn ucb_score(&self, agent_id: &str, step_index: u32, current_round: u32) -> f64 {
        let key = format!("{}.{}", agent_id, step_index);
        let stat = match self.stats.get(&key) {
            Some(s) => s,
            None => return f64::INFINITY, // Unknown step = max priority
        };

        let n_i = stat.attack_count;
        if n_i == 0 {
            return f64::INFINITY; // Never attacked = explore first
        }

        let n = self.total_attacks.max(1) as f64;
        let n_i_f = n_i as f64;

        // Exploitation: historical collapse rate
        let exploitation = stat.collapse_count as f64 / n_i_f;

        // Exploration: UCB1 term
        let exploration = self.exploration_constant * (n.ln() / n_i_f).sqrt();

        // Recency bonus: long-unattacked steps get priority
        let rounds_since = (current_round as i32 - stat.last_attack_round).max(0) as f64;
        let recency = 0.1 * (1.0 + rounds_since).ln();

        exploitation + exploration + recency
    }

    /// Select top-k attack targets for a given attacker
    ///
    /// Excludes:
    /// - Attacker's own steps
    /// - Fully collapsed steps (collapse > 0, defend == 0)
    /// - Fully fortified steps (defend >= 3, collapse == 0)
    pub fn select_targets(
        &self,
        attacker_id: &str,
        current_round: u32,
        top_k: usize,
    ) -> Vec<(String, u32, f64)> {
        let mut candidates: Vec<(String, u32, f64)> = self
            .stats
            .values()
            .filter(|s| {
                s.agent_id != attacker_id
                    && !(s.collapse_count > 0 && s.defend_count == 0)
                    && !(s.defend_count >= 3 && s.collapse_count == 0)
            })
            .map(|s| {
                let score = self.ucb_score(&s.agent_id, s.step_index, current_round);
                (s.agent_id.clone(), s.step_index, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        candidates.truncate(top_k);
        candidates
    }

    /// Get attack coverage ratio
    pub fn coverage(&self) -> f64 {
        if self.stats.is_empty() {
            return 0.0;
        }
        let attacked = self.stats.values().filter(|s| s.attack_count > 0).count();
        attacked as f64 / self.stats.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unexplored_has_infinite_ucb() {
        let mut sched = AttackScheduler::new();
        sched.register_step("agent_a", 1);
        assert_eq!(sched.ucb_score("agent_a", 1, 0), f64::INFINITY);
    }

    #[test]
    fn test_coverage() {
        let mut sched = AttackScheduler::new();
        sched.register_chain("a", 4);
        sched.register_chain("b", 4);
        assert_eq!(sched.coverage(), 0.0);

        sched.record_attack("a", 1, false, 1, None);
        sched.record_attack("b", 2, true, 1, None);
        assert!((sched.coverage() - 0.25).abs() < 0.01); // 2/8
    }

    #[test]
    fn test_select_excludes_self() {
        let mut sched = AttackScheduler::new();
        sched.register_chain("attacker", 3);
        sched.register_chain("target", 3);

        let targets = sched.select_targets("attacker", 0, 10);
        assert!(targets.iter().all(|(agent, _, _)| agent != "attacker"));
    }
}
