//! Decentralized Parallel Voting
//!
//! No privileged nodes. All non-attacker agents vote simultaneously.
//! Majority decision with optional reputation-weighted voting.

use crate::types::Vote;
use crate::reputation::ReputationEngine;

/// Compute majority decision from votes (unweighted)
pub fn majority_decision(votes: &[Vote]) -> (Vote, u32, u32) {
    let collapse = votes.iter().filter(|v| **v == Vote::Collapsed).count() as u32;
    let defend = votes.iter().filter(|v| **v == Vote::Defended).count() as u32;

    let outcome = if collapse > defend {
        Vote::Collapsed
    } else {
        Vote::Defended
    };

    (outcome, collapse, defend)
}

/// Compute weighted majority decision using reputation
pub fn weighted_majority(
    votes: &[(String, Vote)],
    reputation: &ReputationEngine,
) -> (Vote, f64, f64) {
    let mut weighted_collapse = 0.0;
    let mut weighted_defend = 0.0;

    for (agent_id, vote) in votes {
        let weight = reputation.vote_weight(agent_id);
        match vote {
            Vote::Collapsed => weighted_collapse += weight,
            Vote::Defended => weighted_defend += weight,
        }
    }

    let outcome = if weighted_collapse > weighted_defend {
        Vote::Collapsed
    } else {
        Vote::Defended
    };

    (outcome, weighted_collapse, weighted_defend)
}

/// Check Byzantine fault tolerance
///
/// With n voters, tolerates floor((n-1)/2) Byzantine nodes.
pub fn max_byzantine_tolerance(voter_count: usize) -> usize {
    if voter_count == 0 {
        0
    } else {
        (voter_count - 1) / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_collapse() {
        let votes = vec![Vote::Collapsed, Vote::Collapsed, Vote::Defended];
        let (outcome, c, d) = majority_decision(&votes);
        assert_eq!(outcome, Vote::Collapsed);
        assert_eq!(c, 2);
        assert_eq!(d, 1);
    }

    #[test]
    fn test_majority_defend_on_tie() {
        let votes = vec![Vote::Collapsed, Vote::Defended];
        let (outcome, _, _) = majority_decision(&votes);
        assert_eq!(outcome, Vote::Defended); // Tie goes to defend
    }

    #[test]
    fn test_byzantine_tolerance() {
        assert_eq!(max_byzantine_tolerance(3), 1);
        assert_eq!(max_byzantine_tolerance(7), 3);
        assert_eq!(max_byzantine_tolerance(30), 14);
    }

    #[test]
    fn test_weighted_majority() {
        let mut rep = ReputationEngine::new();
        rep.register("strong");
        rep.register("weak");
        // Make "strong" have higher rating
        rep.record_attack("strong", "weak", true);
        rep.record_attack("strong", "weak", true);

        let votes = vec![
            ("strong".to_string(), Vote::Defended),
            ("weak".to_string(), Vote::Collapsed),
        ];

        let (outcome, _, _) = weighted_majority(&votes, &rep);
        assert_eq!(outcome, Vote::Defended); // Strong agent's vote wins
    }
}
